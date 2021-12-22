use anyhow::{anyhow, bail, Context, Result};
use git2::{Commit, IndexAddOption, ObjectType, Oid, Repository, ResetType, Signature};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "smithy-rs-sync")]
/// A CLI tool to replay commits from smithy-rs, generate code, and commit that code to aws-rust-sdk
struct Opt {
    /// The path to the smithy-rs repo folder
    #[structopt(long, parse(from_os_str))]
    smithy_rs: PathBuf,
    /// The path to the aws-sdk-rust folder
    #[structopt(long, parse(from_os_str))]
    aws_sdk: PathBuf,
    /// The branch in aws-sdk-rust that commits will be mirrored to
    #[structopt(long, default_value = "next")]
    branch: String,
}

const BOT_NAME: &str = "AWS SDK Rust Bot";
const BOT_EMAIL: &str = "aws-sdk-rust-primary@amazon.com";

/// Run this app in order to keep aws-sdk-rust in sync with smithy-rs.
///
/// pre-requisites:
/// - an up-to-date local copy of smithy-rs repo
/// - an up-to-date local copy of aws-sdk-rs repo
/// - a Unix-ey system (for the `cp` and `rf` commands to work)
/// - Java Runtime Environment v11 (in order to run gradle commands)
///
/// ```sh
/// cargo run -- \
/// --smithy-rs /Users/zhessler/Documents/smithy-rs-test/ \
/// --aws-sdk /Users/zhessler/Documents/aws-sdk-rust-test/
/// ```
fn main() {
    let Opt {
        smithy_rs,
        aws_sdk,
        branch,
    } = Opt::from_args();

    sync_aws_sdk_with_smithy_rs(&smithy_rs, &aws_sdk, &branch).unwrap();
}

/// Run through all commits made to `smithy-rs` since last sync and "replay" them onto `aws-sdk-rust`
fn sync_aws_sdk_with_smithy_rs(smithy_rs: &Path, aws_sdk: &Path, branch: &str) -> Result<()> {
    let context = "couldn't sync commits";

    // Open the repositories we'll be working with
    let smithy_rs_repo = Repository::open(smithy_rs).context(context).unwrap();
    let aws_sdk_repo = Repository::open(aws_sdk).context(context).unwrap();

    // Check repo that we're going to be moving the code into to see what commit it was last synced with
    let last_synced_commit = get_last_synced_commit(aws_sdk).context(context).unwrap();
    let commit_revs = commits_to_be_applied(&smithy_rs_repo, &last_synced_commit)
        .context(context)
        .unwrap();

    if commit_revs.is_empty() {
        println!("There are no new commits to be applied, have a nice day.");
        return Ok(());
    }

    // `git checkout` the branch of `aws-sdk-rust` that we want to replay commits onto.
    // By default, this is the `next` branch.
    checkout_branch_to_sync_to(&aws_sdk_repo, branch)
        .context(context)
        .unwrap();

    let number_of_commits = commit_revs.len();
    println!("Syncing {} commit(s)...", number_of_commits);
    // Run through all the new commits, syncing them one by one
    for (i, rev) in commit_revs.iter().enumerate() {
        let commit = smithy_rs_repo.find_commit(*rev).context(context).unwrap();

        println!("[{}/{}]\tsyncing {}...", i + 1, number_of_commits, rev);
        checkout_commit_to_sync_from(&smithy_rs_repo, &commit).unwrap();
        build_copy_and_commit_sdk(&commit, &aws_sdk_repo, &smithy_rs_repo)
            .context(context)
            .unwrap();
    }
    println!("Successfully synced {} commit(s)", commit_revs.len());

    // Get the last commit we synced so that we can set that for the next time this tool gets run
    let last_synced_commit = commit_revs
        .last()
        .expect("can't be empty because we'd have early returned");
    println!("updating 'commit at last sync' to {}", last_synced_commit);

    // Update the file containing the commit hash
    set_last_synced_commit(&aws_sdk_repo, last_synced_commit)
        .context(context)
        .unwrap();
    // Commit the file containg the commit hash
    commit_last_synced_commit_file(&aws_sdk_repo)
        .context(context)
        .unwrap();

    println!("All commits have been synced. Don't forget to push them");

    Ok(())
}

/// Starting from a given commit, walk the tree to its `HEAD` in order to build a list of commits that we'll
/// need to sync. If you don't see the commits you're expecting, make sure the repo is up to date.
/// This function doesn't include the `since_commit` in the list since that commit was synced last time
/// this tool was run.
fn commits_to_be_applied(smithy_rs_repo: &Repository, since_commit: &Oid) -> Result<Vec<Oid>> {
    let context = "couldn't create list of commits to be applied";
    let mut commit_revs = Vec::new();

    let rev_range = format!("{}..HEAD", since_commit);
    println!("checking for commits in range {}", rev_range);

    let mut rev_walk = smithy_rs_repo.revwalk().context(context).unwrap();
    rev_walk.push_range(&rev_range).unwrap();

    for rev in rev_walk {
        let rev = rev.unwrap();
        commit_revs.push(rev);
    }

    // Order the revs from earliest to latest
    commit_revs.reverse();

    Ok(commit_revs)
}

const COMMIT_HASH_FILENAME: &str = ".smithyrs-githash";

/// Read the file from aws-sdk-rust that tracks the last smithy-rs commit it was synced with.
/// Returns the hash of that commit.
fn get_last_synced_commit(repo_path: &Path) -> Result<Oid> {
    let path = repo_path.join(COMMIT_HASH_FILENAME);

    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)
        .with_context(|| format!("Couldn't open '{}' file", COMMIT_HASH_FILENAME))
        .unwrap();
    // Commit hashes are 40 chars long
    let mut commit_hash = String::with_capacity(40);
    file.read_to_string(&mut commit_hash)
        .with_context(|| format!("Couldn't read from '{}'", path.to_string_lossy()))
        .unwrap();

    // We trim here in case some really helpful IDE added a newline to the file
    let oid = Oid::from_str(commit_hash.trim())
        .with_context(|| format!("'{}' file didn't contain a valid OID", COMMIT_HASH_FILENAME))
        .unwrap();
    Ok(oid)
}

/// Write the last synced commit to the file in aws-sdk-rust that tracks the last smithy-rs commit it was synced with.
fn set_last_synced_commit(repo: &Repository, oid: &Oid) -> Result<()> {
    let repo_path = repo.workdir().expect("this will always exist");
    let path = repo_path.join(COMMIT_HASH_FILENAME);
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.write(oid.to_string().as_bytes())
        .with_context(|| format!("Couldn't write commit hash to '{}'", path.to_string_lossy()))
        .unwrap();

    Ok(())
}

/// Run the gradle commands to build a fresh SDK in smithy-rs, clean out old SDK files from aws-sdk-rust,
/// copy over the freshly built SDK, and then commit those changes.
fn build_copy_and_commit_sdk(
    commit_to_mirror: &Commit,
    aws_sdk_repo: &Repository,
    smithy_rs_repo: &Repository,
) -> Result<()> {
    let smithy_rs_path = smithy_rs_repo.workdir().expect("this will always exist");
    let aws_sdk_path = aws_sdk_repo.workdir().expect("this will always exist");

    let build_artifacts = build_sdk(smithy_rs_path).unwrap();
    clean_out_existing_sdk(aws_sdk_path).unwrap();
    copy_sdk(&build_artifacts, aws_sdk_path).unwrap();
    create_mirror_commit(aws_sdk_repo, commit_to_mirror).unwrap();

    Ok(())
}

/// Run the necessary commands to build the SDK. On success, returns the path to the folder containing
/// the build artifacts.
fn build_sdk(smithy_rs_path: &Path) -> Result<PathBuf> {
    println!("\tbuilding the SDK...");
    let start = Instant::now();
    let gradlew = smithy_rs_path.join("gradlew");

    // The output of running this command isn't logged anywhere unless it fails
    let clean_command_output = Command::new(&gradlew)
        .arg("-Paws.fullsdk=true")
        .arg(":aws:sdk:clean")
        .current_dir(smithy_rs_path)
        .output()
        .with_context(|| {
            format!(
                "failed to execute '{} -Paws.fullsdk=true :aws:sdk:clean'",
                gradlew.to_string_lossy()
            )
        })
        .unwrap();

    if !clean_command_output.status.success() {
        let stderr = String::from_utf8_lossy(&clean_command_output.stderr);
        let stdout = String::from_utf8_lossy(&clean_command_output.stdout);

        println!("stdout:\n{}\n", stdout);
        println!("stderr:\n{}\n", stderr);

        bail!("failed to clean out repository before assembling the SDK")
    }

    let assemble_command_output = Command::new(&gradlew)
        .arg("-Paws.fullsdk=true")
        .arg(":aws:sdk:assemble")
        .current_dir(smithy_rs_path)
        .output()
        .with_context(|| {
            format!(
                "failed to execute '{} -Paws.fullsdk=true :aws:sdk:assemble'",
                gradlew.to_string_lossy()
            )
        })
        .unwrap();

    if !assemble_command_output.status.success() {
        let stderr = String::from_utf8_lossy(&clean_command_output.stderr);
        let stdout = String::from_utf8_lossy(&clean_command_output.stdout);

        println!("stdout:\n{}\n", stdout);
        println!("stderr:\n{}\n", stderr);

        bail!("failed to assemble the SDK")
    }

    let build_artifact_path = smithy_rs_path.join("aws/sdk/build/aws-sdk");

    println!("\tsuccessfully built the SDK in {:?}", start.elapsed());
    Ok(build_artifact_path)
}

/// Delete any current SDK files in aws-sdk-rust. Run this before copying over new files.
fn clean_out_existing_sdk(aws_sdk_path: &Path) -> Result<()> {
    println!("\tcleaning out previously built SDK...");

    let sdk_path = format!("{}/sdk/*", aws_sdk_path.to_string_lossy());
    let remove_sdk_command_output = Command::new("rm")
        .arg("-rf")
        .arg(&sdk_path)
        .current_dir(aws_sdk_path)
        .output()
        .unwrap();
    if !remove_sdk_command_output.status.success() {
        bail!("failed to clean out the SDK folder at {}", sdk_path);
    }

    let examples_path = format!("{}/example/*", aws_sdk_path.to_string_lossy());
    let remove_examples_command_output = Command::new("rm")
        .arg("-rf")
        .arg(&examples_path)
        .current_dir(aws_sdk_path)
        .output()
        .unwrap();
    if !remove_examples_command_output.status.success() {
        bail!(
            "failed to clean out the examples folder at {}",
            examples_path
        );
    }

    println!("\tsuccessfully cleaned out previously built SDK");
    Ok(())
}

/// Use `cp -r` to recursively copy all files and folders from the smithy-rs build artifacts folder
/// to the aws-sdk-rust repo folder
fn copy_sdk(from_path: &Path, to_path: &Path) -> Result<()> {
    println!("\tcopying built SDK...");

    let copy_sdk_command_output = Command::new("cp")
        .arg("-r")
        .arg(&from_path)
        .arg(&to_path)
        .output()
        .unwrap();
    if !copy_sdk_command_output.status.success() {
        bail!(
            "failed to copy the built SDK from {} to {}",
            from_path.to_string_lossy(),
            to_path.to_string_lossy()
        );
    }

    println!("\tsuccessfully copied built SDK");
    Ok(())
}

/// Find the last commit made to a repo
fn find_last_commit(repo: &Repository) -> Result<Commit> {
    let context = "couldn't find last commit";
    let obj = repo
        .head()
        .context(context)
        .unwrap()
        .resolve()
        .context(context)
        .unwrap()
        .peel(ObjectType::Commit)
        .context(context)
        .unwrap();
    obj.into_commit().map_err(|_| anyhow!(context))
}

/// Create a "mirror" commit. Works by reading a smithy-rs commit and then using the info
/// attached to it to create a commit in aws-sdk-rust.
fn create_mirror_commit(aws_sdk_repo: &Repository, based_on_commit: &Commit) -> Result<()> {
    println!("\tcreating mirror commit...");
    let context = "couldn't create mirror commit";

    let mut index = aws_sdk_repo.index().context(context).unwrap();
    // The equivalent of `git add .`
    index
        .add_all(["."].iter(), IndexAddOption::DEFAULT, None)
        .context(context)
        .unwrap();
    let oid = index.write_tree().context(context).unwrap();
    let parent_commit = find_last_commit(&aws_sdk_repo).context(context).unwrap();
    let tree = aws_sdk_repo.find_tree(oid).context(context).unwrap();

    let _ = aws_sdk_repo
        .commit(
            Some("HEAD"),
            &based_on_commit.author(),
            // TODO maybe we should set this to the name of the bot
            &based_on_commit.committer(),
            based_on_commit.message().unwrap_or_default(),
            &tree,
            &[&parent_commit],
        )
        .context(context)
        .unwrap();

    println!("\tsuccessfully created mirror commit");

    Ok(())
}

/// Commit the file in aws-sdk-rust that tracks what smithy-rs commit the SDK was last built from
fn commit_last_synced_commit_file(aws_sdk_repo: &Repository) -> Result<()> {
    let context = format!("couldn't commit the '{}' file", COMMIT_HASH_FILENAME);

    let mut index = aws_sdk_repo
        .index()
        .with_context(|| context.clone())
        .unwrap();
    index
        .add_path(Path::new(COMMIT_HASH_FILENAME))
        .with_context(|| context.clone())
        .unwrap();
    let signature = Signature::now(BOT_NAME, BOT_EMAIL).unwrap();
    let oid = index.write_tree().with_context(|| context.clone()).unwrap();
    let parent_commit = find_last_commit(&aws_sdk_repo)
        .with_context(|| context.clone())
        .unwrap();
    let tree = aws_sdk_repo
        .find_tree(oid)
        .with_context(|| context.clone())
        .unwrap();

    let _ = aws_sdk_repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("update: {} with last synced commit", COMMIT_HASH_FILENAME),
            &tree,
            &[&parent_commit],
        )
        .context(context)
        .unwrap();

    Ok(())
}

/// `git checkout` the branch of aws-sdk-rust that we want to mirror commits to (defaults to `next`)
fn checkout_branch_to_sync_to(aws_sdk_repo: &Repository, aws_sdk_branch: &str) -> Result<()> {
    let context = "checking out branch to sync to";

    aws_sdk_repo
        .find_remote("origin")?
        .fetch(&["main"], None, None)
        .unwrap();
    let (object, _) = aws_sdk_repo
        .revparse_ext(aws_sdk_branch)
        .context(context)
        .unwrap();

    aws_sdk_repo
        .checkout_tree(&object, None)
        .context(context)
        .unwrap();

    Ok(())
}

/// `git checkout` a commit from smithy-rs that we're going to mirror to aws-sdk-rust
fn checkout_commit_to_sync_from(smithy_rs_repo: &Repository, commit: &Commit) -> Result<()> {
    let head = smithy_rs_repo.head().unwrap().target().unwrap();
    let head = smithy_rs_repo.find_object(head, None).unwrap();
    smithy_rs_repo.reset(&head, ResetType::Hard, None).unwrap();

    smithy_rs_repo
        .checkout_tree(commit.as_object(), None)
        .unwrap();

    Ok(())
}

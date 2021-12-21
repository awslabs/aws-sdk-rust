use anyhow::{bail, Context, Result};
use git2::{Commit, Direction, IndexAddOption, ObjectType, Oid, Repository, Signature, Sort};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "sync")]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    from_repo_at_path: PathBuf,
    #[structopt(long, parse(from_os_str))]
    to_repo_at_path: PathBuf,
}

const BOT_NAME: &str = "AWS SDK Rust Bot";
const BOT_EMAIL: &str = "aws-sdk-rust-primary@amazon.com";
const NEXT_BRANCH: &str = "next";

/*
cargo run -- \
--from-repo-at-path /Users/zhessler/Documents/smithy-rs/ \
--to-repo-at-path /Users/zhessler/Documents/aws-sdk-rust/

when aws-sdk-rust/GENERATED_FROM_THIS_SMITHY_RS_COMMIT
contains 35ac555e0f2c4245b010944290d139afe6753b23

returns:
    commits:
    [
        e38c531464dd4b84b2572fb2831dc0cc95101d01
        a536c15b6e43a424662dd92ff388316ab735d9a5
        bc316a0b81b75a00c389f6281a66eb0f5357172a
        166ffd62f354474af20c5ca6b4737a9ae2cda351
        17818ca7e5221ff1a82356a9b6259f852dc8cb16
        341194e2f91437070a16ceac008e3a85f5aee34f
        2e7ed943513203f1472f2490866dc4fb8a392bd3
        af1832edf62783ba694c8c9f867e6c1c16b3f444
        6325c8a277225e786920d42df935371d27b11fc4
        af6ec39e9f1207e769bdfd34bfdd35fe92fef229
        5ece12393be6c635265a85416efb34534297dfde
    ]
 */

fn main() {
    let Opt {
        from_repo_at_path,
        to_repo_at_path,
    } = Opt::from_args();

    sync_with_latest(&from_repo_at_path, &to_repo_at_path).unwrap();
}

fn sync_with_latest(from_repo_path: &Path, to_repo_path: &Path) -> Result<()> {
    let context = "couldn't sync commits";

    // Check repo that we're going to be moving the code into to see what commit it was last synced with
    let last_synced_commit = get_last_synced_commit(to_repo_path).unwrap();
    let commit_revs = commits_to_be_applied(from_repo_path, &last_synced_commit)
        .context(context)
        .unwrap();

    if commit_revs.is_empty() {
        println!("There are no new commits to be applied, have a nice day.");
        return Ok(());
    }

    let from_repo = Repository::open(from_repo_path).context(context).unwrap();
    let to_repo = Repository::open(to_repo_path).context(context).unwrap();

    println!("commits:");
    for rev in commit_revs.iter() {
        let commit = from_repo.find_commit(*rev).context(context)?;

        println!(
            r#"hash:   {}
author: {}
    {}"#,
            rev,
            commit.author(),
            commit.summary().unwrap_or_default()
        );

        let index_to_check_out = rev.into();
        from_repo
            .checkout_index(Some(index_to_check_out), None)
            .context(context)?;

        build_copy_and_commit_sdk(&commit, from_repo_path, to_repo_path).context(context)?;
    }

    // TODO come up with better name
    let last_synced_commit = commit_revs
        .last()
        .expect("can't be empty because we'd have early returned");
    println!("updating 'commit at last sync' to {}", last_synced_commit);
    // set_last_synced_commit(to_repo_path, last_synced_commit).context(context)?;
    commit_last_synced_commit_file(to_repo_path).context(context)?;

    push_commits(&to_repo, NEXT_BRANCH).context(context)?;

    Ok(())
}

// If you don't see the commits you're expecting, make sure the repo is up to date
// This function doesn't include the `since_commit` in the list
fn commits_to_be_applied(repo_path: &Path, since_commit: &Oid) -> Result<Vec<Oid>> {
    let context = "couldn't create list of commits to be applied";
    let mut commit_revs = Vec::new();

    let repo = Repository::open(repo_path).context(context)?;
    let rev_range = format!("{}..HEAD", since_commit);
    let mut rev_walk = repo.revwalk().context(context)?;
    rev_walk.push_range(&rev_range)?;
    rev_walk.set_sorting(Sort::TOPOLOGICAL)?;

    for rev in rev_walk {
        let rev = rev?;
        commit_revs.push(rev);
    }

    // Order the revs from earliest to latest
    commit_revs.reverse();

    Ok(commit_revs)
}

const COMMIT_HASH_FILENAME: &str = "GENERATED_FROM_THIS_SMITHY_RS_COMMIT";

fn get_last_synced_commit(repo_path: &Path) -> Result<Oid> {
    let path = repo_path.join(COMMIT_HASH_FILENAME);

    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)
        .context("Couldn't open 'GENERATED_FROM_THIS_SMITHY_RS_COMMIT' file")?;
    // Commit hashes are 40 chars long
    let mut commit_hash = String::with_capacity(40);
    file.read_to_string(&mut commit_hash)
        .with_context(|| format!("Couldn't read from '{}'", path.to_string_lossy()))?;

    // We trim here in case some really helpful IDE added a newline to the file
    let oid = Oid::from_str(commit_hash.trim())
        .context("'GENERATED_FROM_THIS_SMITHY_RS_COMMIT' file didn't contain a valid OID")?;

    Ok(oid)
}

fn set_last_synced_commit(repo_path: &Path, oid: Oid) -> Result<()> {
    let path = repo_path.join(COMMIT_HASH_FILENAME);
    let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;

    file.write(oid.to_string().as_bytes())
        .with_context(|| format!("Couldn't write commit hash to '{}'", path.to_string_lossy()))?;

    Ok(())
}

fn build_copy_and_commit_sdk(
    commit_to_mirror: &Commit,
    from_repo_path: &Path,
    to_repo_path: &Path,
) -> Result<()> {
    let build_artifacts = build_sdk(from_repo_path)?;
    clean_out_existing_sdk(to_repo_path)?;
    copy_sdk(&build_artifacts, to_repo_path)?;
    create_mirror_commit(commit_to_mirror, to_repo_path)?;

    Ok(())
}

/// Run the necessary commands to build the SDK, returning the path to the folder containing
/// the build artifacts on success.
fn build_sdk(repo_path: &Path) -> Result<PathBuf> {
    println!("building the SDK...");

    let clean_command_output = Command::new("./gradlew")
        .arg("-Paws.fullsdk=true")
        .arg(":aws:sdk:clean")
        .output()
        .context("failed to execute './gradlew -Paws.fullsdk=true :aws:sdk:clean'")?;

    if !clean_command_output.status.success() {
        bail!("failed to clean out repository before assembling the SDK")
    }

    let assemble_command_output = Command::new("./gradlew")
        .arg("-Paws.fullsdk=true")
        .arg(":aws:sdk:assemble")
        .output()
        .context("failed to execute './gradlew -Paws.fullsdk=true :aws:sdk:assemble'")?;

    if !assemble_command_output.status.success() {
        bail!("failed to assemble the SDK")
    }

    let build_artifact_path = repo_path.join("aws/sdk/build/aws-sdk");

    println!("successfully built the SDK");
    Ok(build_artifact_path)
}

fn clean_out_existing_sdk(repo_path: &Path) -> Result<()> {
    println!("cleaning out previously built SDK...");
    let rm_rf = Command::new("rm").arg("-rf");

    let sdk_path = format!("{}/sdk/*", repo_path.to_string_lossy());
    let remove_sdk_command_output = rm_rf.arg(&sdk_path);
    if !remove_sdk_command_output.status.success() {
        bail!("failed to clean out the SDK folder at {}", sdk_path);
    }

    let examples_path = format!("{}/example/*", repo_path.to_string_lossy());
    let remove_examples_command_output = rm_rf.arg(&examples_path);
    if !remove_examples_command_output.status.success() {
        bail!(
            "failed to clean out the examples folder at {}",
            examples_path
        );
    }

    println!("successfully cleaned out previously built SDK");
    Ok(())
}

fn copy_sdk(from_path: &Path, to_path: &Path) -> Result<()> {
    println!("copying built SDK...");

    let copy_sdk_command_output = Command::new("cp").arg("-r").arg(&from_path).arg(&to_path);
    if !copy_sdk_command_output.status.success() {
        bail!(
            "failed to copy the built SDK from {} to {}",
            from_path.to_string_lossy(),
            to_path.to_string_lossy()
        );
    }

    println!("successfully copied built SDK");
    Ok(())
}

fn find_last_commit(repo: &Repository) -> Result<Commit> {
    let context = "couldn't find last commit";
    let obj = repo
        .head()
        .context(context)?
        .resolve()
        .context(context)?
        .peel(ObjectType::Commit)
        .context(context)?;
    obj.into_commit().context("couldn't find last commit")
}

fn create_mirror_commit(based_on_commit: &Commit, repo_path: &Path) -> Result<()> {
    let context = "couldn't create mirror commit";
    let repo = Repository::open(repo_path).context(context)?;
    let mut index = repo.index().context(context)?;
    // The equivalent of `git add .`
    index
        .add_all(["."].iter(), IndexAddOption::DEFAULT, None)
        .context(context)?;
    let oid = index.write_tree().context(context)?;
    let parent_commit = find_last_commit(&repo).context(context)?;
    let tree = repo.find_tree(oid).context(context)?;

    let _ = repo
        .commit(
            Some("HEAD"),
            based_on_commit.signature(),
            // TODO maybe we should set this to the name of the bot
            based_on_commit.signature(),
            based_on_commit.message().unwrap_or_default(),
            &tree,
            &[&parent_commit],
        )
        .context(context)?;

    Ok(())
}

fn commit_last_synced_commit_file(repo_path: &Path) -> Result<()> {
    let context = "couldn't commit the GENERATED_FROM_THIS_SMITHY_RS_COMMIT file";
    let repo = Repository::open(repo_path).context(context)?;
    let mut index = repo.index().context(context)?;
    index
        .add_path(&repo_path.join(COMMIT_HASH_FILENAME))
        .context(context)?;
    let signature = Signature::now(BOT_NAME, BOT_EMAIL)?;
    let oid = index.write_tree().context(context)?;
    let parent_commit = find_last_commit(&repo).context(context)?;
    let tree = repo.find_tree(oid).context(context)?;

    let _ = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "update: GENERATED_FROM_THIS_SMITHY_RS_COMMIT with last synced commit",
            &tree,
            &[&parent_commit],
        )
        .context(context)?;

    Ok(())
}

fn push_commits(repo: &Repository, branch: &str) -> Result<()> {
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote("origin", url)?,
    };
    remote.connect(Direction::Push)?;
    // TODO(https://github.com/rust-lang/rust/issues/67984) Formatting can be simplified even more once linked PR is fully implemented
    let push_to = format!("refs/heads/{b}:refs/heads/{b}", b = branch);
    repo.push(&[&push_to], None)
}
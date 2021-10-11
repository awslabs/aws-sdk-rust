/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use anyhow::{bail, Context};
use aws_sdk_polly::model::{Engine, OutputFormat, VoiceId};
use aws_sdk_transcribe::model::{LanguageCode, Media, MediaFormat};
use clap::{crate_authors, crate_description, crate_name, crate_version, ArgMatches};
use rodio::{Decoder, OutputStream, Sink};
use std::time::Duration;
use tempdir::TempDir;
use tokio::{io::AsyncWriteExt, task::spawn_blocking};
use tracing::{debug, error, info};

/// While playing the telephone game, the user can pass an arg that defines how many times to pass the message through Polly and Transcribe.
/// This is the default number of iterations to do when the user doesn't specify.
const DEFAULT_NUMBER_OF_ITERATIONS: u32 = 5;
/// When running a job/task that takes some time to complete (speech synthesis and transcription), this sets a maximum wait time in seconds before giving up.
const TASK_TIMEOUT_IN_SECONDS: i32 = 30;
/// How often to poll for job/task status
const TASK_WAIT_INTERVAL_IN_SECONDS: i32 = 2;

#[tokio::main]
async fn main() {
    // By default, hide any message that isn't an error unless it's from the game
    let rust_log =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "error,telephone_game=debug".to_owned());
    //  Start up the logger
    tracing_subscriber::fmt().with_env_filter(rust_log).init();
    let app = build_clap_app();

    let res = match app.get_matches().subcommand() {
        ("play", Some(matches)) => play_telephone(matches).await,
        ("polly", Some(matches)) => test_polly(matches).await,
        _ => unreachable!(),
    };

    if let Err(e) = res {
        let error_chain: String = e
            .chain()
            // We skip the first error so it doesn't get printed twice
            .skip(1)
            .map(|e| format!("Caused by:\n\t{}\n", e))
            .collect();
        let full_error_message = format!("Encountered an error: {}\n{}", e, error_chain);

        error!("{}", full_error_message);
    }
}

fn build_clap_app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
        clap::App::new("play")
                .about("Start playing a game of Telephone")
                .args_from_usage("--phrase=[PHRASE] 'The phrase to play the game with'")
                .args_from_usage("--iterations=[ITERATIONS] 'The number of times to relay the telephone message, defaults to 1 when omitted'")
                .args_from_usage("--bucket-name=[BUCKET_NAME] 'The name of the S3 bucket that will be used to store intermediate audio and text files created by the game, defaults to telephone-game when omitted'")
        )
        .subcommand(clap::App::new("polly").about("Make Polly say something")
        .args_from_usage("--phrase=[PHRASE] 'The phrase you want Polly to say'")
    )
}

/// Make Polly speak what you type
async fn test_polly(matches: &ArgMatches<'_>) -> anyhow::Result<()> {
    let phrase = matches
        .value_of("phrase")
        .context("You must pass a phrase")?;

    info!("Making Polly say '{}'", phrase);

    // Create a new AWS Config
    let config = aws_config::load_from_env().await;
    let polly_client = aws_sdk_polly::Client::new(&config);

    // Set up a temp directory to store audio files
    let tmp_dir = TempDir::new("telephone-game").expect("couldn't create temp dir");
    let tmp_file_path = tmp_dir.path().join("polly.mp3");

    // Start synthesizing speech
    let res = polly_client
        .synthesize_speech()
        .text(phrase)
        .voice_id(VoiceId::Joanna)
        .output_format(OutputFormat::Mp3)
        .send()
        .await
        .context("Failed to synthesize your phrase into speech")?;

    info!("Playing Polly's response...");

    // Collect the ByteStream returned by the synthesize_speech call
    let byte_stream = res
        .audio_stream
        .collect()
        .await
        .context("Audio stream ended prematurely")?;

    // Create a file to store the audio
    let mut tmp_file = tokio::fs::File::create(&tmp_file_path)
        .await
        .context("Failed to create temp file")?;
    // Write the ByteStream to the file
    tmp_file
        .write_all(&byte_stream.into_bytes())
        .await
        .context("Failed to write to temp file")?;
    // Flush the write operation to ensure it finishes before we continue
    tmp_file
        .flush()
        .await
        .context("Failed to flush after writing file")?;

    spawn_blocking(move || {
        // Set up the ability to play audio
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Couldn't get handle to default audio output");
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Open the audio file with regular blocking IO File
        // rodio's Decoder requires stdlib Files
        let file = std::fs::File::open(&tmp_file_path).context("Failed to re-open audio file")?;
        let source =
            Decoder::new(std::io::BufReader::new(file)).context("Failed to decode audio")?;

        // Set rodio to play the audio we just decoded
        sink.append(source);
        sink.sleep_until_end();

        Ok::<(), anyhow::Error>(())
    })
    // Yes, two are necessary: one for the functions in the closure, one for spawn_blocking
    .await??;

    info!("Did you hear it?");

    Ok(())
}

/// Play a game of Telephone w/ AWS
async fn play_telephone(matches: &ArgMatches<'_>) -> anyhow::Result<()> {
    // Fetch user any user input that will override default values
    let number_of_iterations = matches
        .value_of("iterations")
        .and_then(|i| i.parse::<u32>().ok())
        .unwrap_or(DEFAULT_NUMBER_OF_ITERATIONS);
    if number_of_iterations == 0 {
        bail!("Iterations must be a number greater than 0");
    }

    let original_phrase = matches.value_of("phrase").unwrap_or_default();
    let mut current_phrase = original_phrase.to_owned();

    let bucket_name = matches
        .value_of("bucket-name")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "telephone-game".to_owned());

    // Create a config and required clients for AWS services
    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let polly_client = aws_sdk_polly::Client::new(&config);
    let transcribe_client = aws_sdk_transcribe::Client::new(&config);

    // Create a bucket to store audio and transcriptions if none exists
    let bucket_name = create_s3_bucket_if_not_exists(&s3_client, &bucket_name)
        .await
        .context("Failed to complete necessary setup")?;

    for i in 0..number_of_iterations {
        debug!(
            "starting speech synthesis task for phrase '{}' ({} iterations)",
            &current_phrase, &number_of_iterations
        );

        // Start a speech synthesis task and set it to output to the previously created S3 bucket
        let output_uri = synthesize_speech(&polly_client, &current_phrase, &bucket_name).await?;

        // Job names must be unique so we clear the old job to reuse the name.
        delete_transcription_job(&transcribe_client, "telephone-game-transcription").await;

        // Transcribe the speech file generated previously
        transcribe_speech(
            &transcribe_client,
            "telephone-game-transcription",
            &output_uri,
            &bucket_name,
        )
        .await?;

        // Download the transcription from S3 and parse out the full transcription text
        let transcript =
            get_transcript_from_s3(&s3_client, "telephone-game-transcription", &bucket_name)
                .await?;

        info!("Transcription #{} == {}", i, &transcript);
        current_phrase = transcript;
    }

    // Log the final output
    info!(
        r#"The phrase
"{}"
became
"{}"
after {} iterations"#,
        original_phrase, current_phrase, number_of_iterations
    );

    Ok(())
}

// Start a speech synthesis job and wait until it finishes before returning the URI of the audio file
async fn synthesize_speech(
    polly_client: &aws_sdk_polly::Client,
    input_text: &str,
    output_bucket_name: &str,
) -> Result<String, anyhow::Error> {
    let mut synthesis_task = polly_client
        .start_speech_synthesis_task()
        .text(input_text)
        .voice_id(VoiceId::Joanna)
        .output_format(OutputFormat::Mp3)
        .output_s3_bucket_name(output_bucket_name)
        .engine(Engine::Standard)
        .send()
        .await
        .context("Failed to start speech synthesis task")?
        .synthesis_task
        .unwrap();

    debug!(
        "Waiting for speech synthesis task to complete. Timeout is {}s",
        TASK_TIMEOUT_IN_SECONDS
    );

    let mut speech_synthesis_timeout_in_seconds = TASK_TIMEOUT_IN_SECONDS;

    // Wait up to TASK_TIMEOUT_IN_SECONDS seconds for synthesis task to complete
    // The status of the task is checked every TASK_WAIT_INTERVAL_IN_SECONDS in a loop
    // Break out of the loop once the task succeeds or fails
    'synthesis_task: loop {
        speech_synthesis_timeout_in_seconds -= TASK_WAIT_INTERVAL_IN_SECONDS;
        tokio::time::sleep(Duration::from_secs(TASK_WAIT_INTERVAL_IN_SECONDS as u64)).await;
        synthesis_task = polly_client
            .get_speech_synthesis_task()
            .task_id(synthesis_task.task_id.unwrap())
            .send()
            .await
            .context("Failed to check status of speech synthesis task")?
            .synthesis_task
            .unwrap();

        use aws_sdk_polly::model::TaskStatus;
        match synthesis_task.task_status.unwrap() {
            TaskStatus::Completed => {
                debug!("Speech synthesis task completed");
                break 'synthesis_task;
            }
            TaskStatus::Failed => {
                let reason = synthesis_task
                    .task_status_reason
                    .unwrap_or_else(|| "(no reason given)".to_owned());

                bail!("Speech synthesis task failed with reason: {}", reason);
            }
            TaskStatus::InProgress | TaskStatus::Scheduled => {
                debug!("Speech synthesis is ongoing...")
            }
            // New TaskStatus variants could get added in the future. It's always a good idea to handle this case with a helpful message
            unknown => bail!("Failed to handle unknown task status {:?}", unknown),
        }

        if speech_synthesis_timeout_in_seconds <= 0 {
            bail!(
                "Speech synthesis task didn't complete before the {}s timeout elapsed",
                TASK_TIMEOUT_IN_SECONDS
            );
        }
    }

    Ok(synthesis_task.output_uri.unwrap())
}

// Delete a transcription job. If no job exists with a given name, do nothing
async fn delete_transcription_job(transcribe_client: &aws_sdk_transcribe::Client, job_name: &str) {
    debug!("Clearing pre-existing transcription job");

    match transcribe_client
        .delete_transcription_job()
        .transcription_job_name(job_name)
        .send()
        .await
    {
        Ok(_) => debug!("Previous transcription job deleted"),
        Err(e) => debug!("No previous transcription exists {}", e),
    };
}

// Start a transcription job and wait until it finishes before returning
async fn transcribe_speech(
    transcribe_client: &aws_sdk_transcribe::Client,
    job_name: &str,
    media_file_uri: &str,
    output_bucket_name: &str,
) -> Result<(), anyhow::Error> {
    let media = Media::builder().media_file_uri(media_file_uri).build();

    let mut transcription_job = transcribe_client
        .start_transcription_job()
        .transcription_job_name(job_name)
        .media_format(MediaFormat::Mp3)
        .language_code(LanguageCode::EnUs)
        .media(media)
        .output_bucket_name(output_bucket_name)
        .send()
        .await
        .context("Failed to start transcription job")?
        .transcription_job
        .unwrap();

    debug!(
        "Waiting for transcription job to complete. Timeout is {}s",
        TASK_TIMEOUT_IN_SECONDS
    );

    let mut transcription_job_timeout_in_seconds = TASK_TIMEOUT_IN_SECONDS;

    // Wait up to TASK_TIMEOUT_IN_SECONDS seconds for transcription job to complete
    // The status of the job is checked every TASK_WAIT_INTERVAL_IN_SECONDS in a loop
    // Break out of the loop once the job succeeds or fails
    'transcription_job: loop {
        transcription_job_timeout_in_seconds -= TASK_WAIT_INTERVAL_IN_SECONDS;
        tokio::time::sleep(Duration::from_secs(TASK_WAIT_INTERVAL_IN_SECONDS as u64)).await;

        transcription_job = transcribe_client
            .get_transcription_job()
            .transcription_job_name(transcription_job.transcription_job_name.unwrap())
            .send()
            .await
            .context("Failed to check status of transcription job")?
            .transcription_job
            .unwrap();

        use aws_sdk_transcribe::model::TranscriptionJobStatus;
        match transcription_job.transcription_job_status.unwrap() {
            TranscriptionJobStatus::Completed => {
                debug!("Transcription job completed");
                break 'transcription_job;
            }
            TranscriptionJobStatus::Failed => {
                let reason = transcription_job
                    .failure_reason
                    .unwrap_or_else(|| "(no reason given)".to_owned());
                bail!("Transcription job failed with reason: {}", reason);
            }
            TranscriptionJobStatus::InProgress | TranscriptionJobStatus::Queued => {
                debug!("Transcription job is ongoing...")
            }
            // New TranscriptionJobStatus variants could get added in the future. It's always a good idea to handle this case with a helpful message
            unknown => bail!(
                "Failed to handle unknown transcription job status {:?}",
                unknown
            ),
        }

        if transcription_job_timeout_in_seconds <= 0 {
            bail!(
                "Transcription job didn't complete before the {}s timeout elapsed",
                TASK_TIMEOUT_IN_SECONDS
            );
        }
    }

    Ok(())
}

/// Download the transcript JSON file from S3 and output the transcription
async fn get_transcript_from_s3(
    s3_client: &aws_sdk_s3::Client,
    transcription_job_name: &str,
    bucket_containing_transcript: &str,
) -> Result<String, anyhow::Error> {
    let transcription_file_name = format!("{}.json", transcription_job_name);
    let get_object_output = s3_client
        .get_object()
        .bucket(bucket_containing_transcript)
        .key(&transcription_file_name)
        .send()
        .await
        .context("Failed to get transcript from S3")?;

    let body = get_object_output
        .body
        .collect()
        .await
        .context("Failed to collect ByteStream")?
        .into_bytes();

    let transcript =
        std::str::from_utf8(&body).context("Failed to parse transcript as UTF-8 text")?;
    let transcript: serde_json::Value =
        serde_json::from_str(transcript).context("Failed to parse transcript as JSON")?;

    let transcript = transcript["results"]["transcripts"][0]["transcript"]
        .as_str()
        .unwrap()
        .to_owned();

    Ok(transcript)
}

/// Check if a bucket exists and create one if it doesn't. Then, return the bucket's name.
async fn create_s3_bucket_if_not_exists(
    s3_client: &aws_sdk_s3::Client,
    bucket_name: &str,
) -> anyhow::Result<String> {
    let bucket_list = s3_client
        .list_buckets()
        .send()
        .await
        .context("Failed to list buckets when checking for existing bucket")?;
    let maybe_existing_bucket = bucket_list.buckets.unwrap().into_iter().find(|bucket| {
        bucket
            .name
            .as_ref()
            .map(|name| name == bucket_name)
            .unwrap_or_default()
    });

    if let Some(_bucket) = maybe_existing_bucket {
        debug!("A bucket named '{}' already exists", bucket_name);
        Ok(bucket_name.to_owned())
    } else {
        debug!("Creating an S3 bucket to store intermediate text and audio files");
        s3_client
            .create_bucket()
            .bucket(bucket_name)
            .send()
            .await
            .map(|_| {
                debug!("Created new bucket '{}'", bucket_name);
                bucket_name.to_owned()
            })
            .with_context(|| format!("Failed to create new bucket '{}'", bucket_name))
    }
}

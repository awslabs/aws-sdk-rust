#!/usr/bin/env python3
#  Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
#  SPDX-License-Identifier: Apache-2.0

import sys
import os
import os.path as path
from pathlib import Path

# Ensure working directory is the script path
script_path = path.dirname(path.realpath(__file__))


# Looks for aws-models in the parent directory of aws-sdk-rust
def discover_aws_models():
    repo_path = path.abspath(path.join(script_path, "../aws-models"))
    git_path = repo_path + "/.git"
    if path.exists(repo_path) and path.exists(git_path):
        print(f"Discovered aws-models at {repo_path}")
        return repo_path
    else:
        return None


def discover_new_models(aws_models_repo, known_models):
    new_models = []
    for model in os.listdir(aws_models_repo):
        if model not in known_models and path.exists(Path(aws_models_repo) / model / "smithy" / "model.json"):
            new_models.append(model)
    return new_models


def copy_model(source_path, model_path):
    dest_path = Path("aws-models") / model_path
    source = source_path.read_text()
    # Add a newline at the end when copying the model over
    with open(dest_path, "w") as file:
        file.write(source)
        if not source.endswith("\n"):
            file.write("\n")


def copy_known_models(aws_models_repo):
    known_models = set()
    for model in os.listdir("aws-models"):
        if not model.endswith('.json'):
            continue
        model_name = model[:-len('.json')]
        known_models.add(model_name)
        source_path = Path(aws_models_repo) / model_name / "smithy" / "model.json"
        if not source_path.exists():
            print(f"  Warning: cannot find model for '{model_name}' in aws-models, but it exists in aws-sdk-rust")
            continue
        copy_model(source_path, model)
    return known_models


def main():
    # Acquire model location
    aws_models_repo = discover_aws_models()
    if aws_models_repo is None:
        if len(sys.argv) != 2:
            print("Please provide the location of the aws-models repository as the first argument")
            sys.exit(1)
        else:
            aws_models_repo = sys.argv[1]

    print("Copying over known models...")
    known_models = copy_known_models(aws_models_repo)

    print("Looking for new models...")
    new_models = discover_new_models(aws_models_repo, known_models)
    if len(new_models) > 0:
        print(f"  Warning: found models for {new_models} in aws-models that aren't in aws-sdk-rust")
        print("  Run the following commands to bring these in:\n")
        for model in new_models:
            print(f"  touch aws-models/{model}.json")
        print("  ./sync-models.py\n")

    print("Copying endpoints.json...")
    copy_model(Path(aws_models_repo) / "endpoints.json", "sdk-endpoints.json")
    print("Copying default-configuration.json...")
    copy_model(Path(aws_models_repo) / "default-configuration.json", "sdk-default-configuration.json")

    print("Models synced.")


if __name__ == "__main__":
    main()

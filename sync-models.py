#!/usr/bin/env python3
#  Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
#  SPDX-License-Identifier: Apache-2.0

import sys
import os
import os.path as path
from pathlib import Path

# Ensure working directory is the script path
script_path = path.dirname(path.realpath(__file__))


def discover_repo(name, relative_path):
    """Look for a repo at a relative path from this script."""
    repo_path = path.abspath(path.join(script_path, relative_path))
    git_path = repo_path + "/.git"
    if path.exists(repo_path) and path.exists(git_path):
        print(f"Discovered {name} at {repo_path}")
        return repo_path
    return None


def discover_aws_models():
    """Auto-discover a models repo in common locations."""
    # Try public api-models-aws first (preferred)
    repo = discover_repo("api-models-aws", "../api-models-aws")
    if repo:
        return repo, "public"
    # Try private aws-models (legacy)
    repo = discover_repo("aws-models", "../aws-models")
    if repo:
        return repo, "private"
    return None, None


def find_smithy_model(repo_path, repo_type, model_name):
    """Find the Smithy model JSON file for a given service name."""
    if repo_type == "public":
        # Public layout: models/<service>/service/<version>/<service>-<version>.json
        service_dir = Path(repo_path) / "models" / model_name / "service"
        if not service_dir.exists():
            return None
        # There's exactly one version directory per service
        versions = [d for d in service_dir.iterdir() if d.is_dir()]
        if not versions:
            return None
        version_dir = versions[0]
        # Find the .json file in the version directory
        json_files = list(version_dir.glob("*.json"))
        if not json_files:
            return None
        return json_files[0]
    else:
        # Private layout: <service>/smithy/model.json
        source_path = Path(repo_path) / model_name / "smithy" / "model.json"
        return source_path if source_path.exists() else None


def get_models_root(repo_path, repo_type):
    """Get the directory containing service model directories."""
    if repo_type == "public":
        return Path(repo_path) / "models"
    else:
        return Path(repo_path)


def discover_new_models(repo_path, repo_type, known_models):
    models_root = get_models_root(repo_path, repo_type)
    new_models = []
    for model in os.listdir(models_root):
        model_path = models_root / model
        if not model_path.is_dir():
            continue
        if model not in known_models and find_smithy_model(repo_path, repo_type, model) is not None:
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


def copy_known_models(repo_path, repo_type):
    # These are global config files, not service models
    SKIP_PREFIXES = ("sdk-",)
    known_models = set()
    for model in os.listdir("aws-models"):
        if not model.endswith('.json'):
            continue
        model_name = model[:-len('.json')]
        if any(model_name.startswith(p) for p in SKIP_PREFIXES):
            continue
        known_models.add(model_name)
        source_path = find_smithy_model(repo_path, repo_type, model_name)
        if source_path is None:
            print(f"  Warning: cannot find model for '{model_name}' in models repo, but it exists in aws-sdk-rust")
            continue
        copy_model(source_path, model)
    return known_models


def copy_global_file(repo_path, repo_type, source_name, dest_name):
    """Copy a global file (endpoints, partitions, default-configuration)."""
    if repo_type == "public":
        # Public repo doesn't have these global files at the root
        # They may be in a different location or not available
        source_path = Path(repo_path) / source_name
        if not source_path.exists():
            # Try models/ subdirectory
            source_path = Path(repo_path) / "models" / source_name
        if not source_path.exists():
            print(f"  Warning: {source_name} not found in public repo, skipping")
            return
    else:
        source_path = Path(repo_path) / source_name

    if source_path.exists():
        copy_model(source_path, dest_name)
    else:
        print(f"  Warning: {source_name} not found, skipping")


def main():
    os.chdir(script_path)

    # Acquire model location
    repo_path, repo_type = discover_aws_models()
    if repo_path is None:
        if len(sys.argv) < 2:
            print("Usage: sync-models.py <path-to-models-repo> [--public|--private]")
            print("")
            print("Supports two repo layouts:")
            print("  --public   github.com/aws/api-models-aws (models/<svc>/service/<ver>/<svc>-<ver>.json)")
            print("  --private  internal aws-models (<svc>/smithy/model.json)")
            print("")
            print("If not specified, auto-detects based on directory structure.")
            sys.exit(1)
        repo_path = sys.argv[1]
        # Auto-detect or use flag
        if "--public" in sys.argv:
            repo_type = "public"
        elif "--private" in sys.argv:
            repo_type = "private"
        elif path.exists(path.join(repo_path, "models")):
            repo_type = "public"
            print(f"Auto-detected public repo layout")
        else:
            repo_type = "private"
            print(f"Auto-detected private repo layout")

    print(f"Using {repo_type} repo layout from {repo_path}")
    print("Copying over known models...")
    known_models = copy_known_models(repo_path, repo_type)

    print("Looking for new models...")
    new_models = discover_new_models(repo_path, repo_type, known_models)
    if len(new_models) > 0:
        print(f"  Warning: found models for {new_models} in models repo that aren't in aws-sdk-rust")
        print("  Run the following commands to bring these in:\n")
        for model in new_models:
            print(f"  touch aws-models/{model}.json")
        print("  ./sync-models.py\n")

    print("Copying endpoints.json...")
    copy_global_file(repo_path, repo_type, "endpoints.json", "sdk-endpoints.json")
    print("Copying default-configuration.json...")
    copy_global_file(repo_path, repo_type, "default-configuration.json", "sdk-default-configuration.json")

    print("Models synced.")


if __name__ == "__main__":
    main()

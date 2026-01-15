#!/usr/bin/env python3
"""
Aether All-in-One Publish Script (v0.1.5)
Automates publishing to Crates.io, PyPI, and NPM.
"""

import os
import sys
import subprocess
import time
from pathlib import Path

# --- Constants & Colors ---
COLORS = {
    "HEADER": "\033[95m",
    "INFO": "\033[94m",
    "SUCCESS": "\033[92m",
    "WARNING": "\033[93m",
    "ERROR": "\033[91m",
    "ENDC": "\033[0m",
    "BOLD": "\033[1m",
}

def log(msg, level="INFO"):
    color = COLORS.get(level, COLORS["INFO"])
    print(f"{color}{COLORS['BOLD'] if level == 'HEADER' else ''}{msg}{COLORS['ENDC']}")

def run_cmd(args, cwd=None, env=None, capture_output=False):
    """Run a shell command and handle errors."""
    try:
        current_env = os.environ.copy()
        if env:
            current_env.update(env)
        
        result = subprocess.run(
            args,
            cwd=cwd,
            env=current_env,
            check=True,
            text=True,
            capture_output=capture_output,
            shell=True if os.name == 'nt' else False
        )
        return result
    except subprocess.CalledProcessError as e:
        log(f"Command failed: {' '.join(args)}", "ERROR")
        if e.stderr:
            print(e.stderr)
        raise

# --- Core Functions ---

def load_env():
    """Load variables from .env file into os.environ."""
    # Check scripts directory first, then project root
    script_dir = Path(__file__).resolve().parent
    root_dir = script_dir.parent
    
    possible_paths = [script_dir / ".env", root_dir / ".env"]
    env_path = next((p for p in possible_paths if p.exists()), None)

    if not env_path:
        log(f"No .env file found in {script_dir} or {root_dir}. Please create it.", "ERROR")
        sys.exit(1)
    
    log(f"Loading environment variables from {env_path}...", "INFO")
    with open(env_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, value = line.split("=", 1)
                os.environ[key.strip()] = value.strip()

def publish_rust():
    log("\n--- Publishing Rust Crates to Crates.io ---", "HEADER")
    token = os.environ.get("CARGO_REGISTRY_TOKEN")
    if not token:
        raise ValueError("CARGO_REGISTRY_TOKEN not found in .env")

    crates = [
        "crates/aether-macros",
        "crates/aether-core",
        "crates/aether-ai",
        "crates/aether-inspector",
        "crates/aether-cli"
    ]

    for crate_path in crates:
        for attempt in range(4):
            log(f"Publishing {crate_path} (Attempt {attempt+1}/4)...", "WARNING")
            try:
                # Capture output to check for "already exists" error
                result = subprocess.run(
                    ["cargo", "publish", "--token", token, "--allow-dirty", "--no-verify"],
                    cwd=crate_path,
                    capture_output=True,
                    text=True,
                    shell=True if os.name == 'nt' else False
                )
                
                if result.returncode == 0:
                    log(f"Successfully published {crate_path}.", "SUCCESS")
                    log("Waiting 45s for Crates.io propagation...", "INFO")
                    time.sleep(45)
                    break # Success, move to next crate
                elif "already exists" in result.stderr or "already exists" in result.stdout:
                    log(f"Crate {crate_path} already exists on Crates.io. Skipping...", "INFO")
                    break # Already done, move to next
                else:
                    if attempt < 3:
                        log(f"Failed to publish {crate_path}. Retrying in 20s...", "WARNING")
                        time.sleep(20)
                        continue
                    
                    log(f"Failed to publish {crate_path} (Exit code {result.returncode})", "ERROR")
                    print(result.stderr or result.stdout)
                    raise subprocess.CalledProcessError(result.returncode, result.args, result.stdout, result.stderr)
            except Exception as e:
                if attempt < 3: 
                    continue
                log(f"Failed to publish {crate_path}: {e}", "ERROR")
                raise

def publish_python():
    log("\n--- Publishing Python Wheel to PyPI ---", "HEADER")
    token = os.environ.get("PYPI_API_TOKEN")
    python_path = os.environ.get("PYTHON_PATH", sys.executable)
    
    if not token:
        raise ValueError("PYPI_API_TOKEN not found in .env")

    os.environ["PYO3_PYTHON"] = python_path
    log(f"Using Python interpreter: {python_path}", "INFO")

    log("Building and publishing aether-python via Maturin...", "WARNING")
    # Set MATURIN_PYPI_TOKEN instead of using --token
    maturin_env = {"MATURIN_PYPI_TOKEN": token}
    run_cmd(["maturin", "publish", "--interpreter", python_path], cwd="crates/aether-python", env=maturin_env)

def publish_node():
    log("\n--- Publishing Node.js Package to NPM ---", "HEADER")
    token = os.environ.get("NPM_TOKEN")
    if not token:
        raise ValueError("NPM_TOKEN not found in .env")

    log("Building NAPI binary...", "WARNING")
    run_cmd(["npm", "run", "build", "--", "--release"], cwd="crates/aether-node")

    log("Setting up NPM authentication...", "INFO")
    # This writes to local .npmrc for the session
    npmrc_path = Path("crates/aether-node/.npmrc")
    with open(npmrc_path, "w") as f:
        f.write(f"//registry.npmjs.org/:_authToken={token}\n")

    log("Publishing to NPM...", "WARNING")
    try:
        run_cmd(["npm", "publish", "--access", "public"], cwd="crates/aether-node")
    finally:
        if npmrc_path.exists():
            npmrc_path.unlink()

# --- Main ---

if __name__ == "__main__":
    start_time = time.time()
    try:
        # Move to root directory
        os.chdir(Path(__file__).parent.parent)
        
        load_env()
        
        log("🚀 Starting Aether v0.1.5 Deployment Stack", "SUCCESS")
        
        # publish_rust()
        publish_python()
        publish_node()
        
        duration = int(time.time() - start_time)
        log(f"\n✨ All packages published successfully in {duration}s!", "SUCCESS")

    except Exception as e:
        log(f"\n❌ Deployment failed: {e}", "ERROR")
        sys.exit(1)

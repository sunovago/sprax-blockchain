"""Read chain CI results or dependency alerts using existing Git credentials.

Credentials stay in memory and are never printed or written to disk.
Usage: python scripts/github_verification.py runs | jobs RUN_ID | logs RUN_ID | alerts
"""
import base64
import io
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
import zipfile

API = "https://api.github.com/repos/sunovago/sprax-blockchain"
HEADERS = {"Accept": "application/vnd.github+json", "User-Agent": "sprax-verification"}


def authenticate():
    env = dict(os.environ, GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="never")
    result = subprocess.run(
        ["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
        text=True, capture_output=True, env=env, check=False,
    )
    values = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)
    if result.returncode or not values.get("password"):
        raise RuntimeError("Existing GitHub credentials are unavailable")
    token = base64.b64encode((values.get("username", "x-access-token") + ":" + values["password"]).encode()).decode()
    HEADERS["Authorization"] = "Basic " + token


def request(path):
    req = urllib.request.Request(API + path, headers=HEADERS)
    try:
        return urllib.request.urlopen(req, timeout=30).read()
    except urllib.error.HTTPError as error:
        if error.code not in (401, 403, 404) or "Authorization" in HEADERS:
            raise RuntimeError(f"GitHub API returned HTTP {error.code}") from None
        authenticate()
        return request(path)


def main():
    mode = sys.argv[1]
    if mode == "runs":
        data = json.loads(request("/actions/runs?branch=codex/mainnet-hardening-20261004&per_page=5"))
        print(json.dumps([{k: r.get(k) for k in ("id", "name", "status", "conclusion", "head_sha", "html_url")} for r in data["workflow_runs"]]))
    elif mode == "jobs":
        print(request(f"/actions/runs/{int(sys.argv[2])}/jobs").decode())
    elif mode == "job-logs":
        print(request(f"/actions/jobs/{int(sys.argv[2])}/logs").decode(errors="replace")[-24000:])
    elif mode == "logs":
        with zipfile.ZipFile(io.BytesIO(request(f"/actions/runs/{int(sys.argv[2])}/logs"))) as archive:
            for name in archive.namelist():
                if name.endswith(".txt") and ("regression" in name.lower() or "lint" in name.lower() or "npm run build" in name.lower()):
                    print(name)
                    print(archive.read(name).decode(errors="replace")[-24000:])
    elif mode == "alerts":
        authenticate()
        data = json.loads(request("/dependabot/alerts?state=open&per_page=100"))
        print(json.dumps([{"number": a["number"], "severity": a["security_advisory"]["severity"], "package": a["dependency"]["package"]["name"], "manifest": a["dependency"]["manifest_path"], "summary": a["security_advisory"]["summary"], "vulnerable_range": a["security_vulnerability"]["vulnerable_version_range"], "patched": a["security_vulnerability"]["first_patched_version"]} for a in data]))
    else:
        raise RuntimeError("Use runs, jobs RUN_ID, logs RUN_ID, or alerts")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, urllib.error.URLError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)

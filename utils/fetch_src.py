import re
from urllib.parse import urljoin
import requests
import os
from datetime import datetime

BASE_URL = "https://www.desmos.com/calculator"
OUTPUT_PREFIX = "fetched_"

def fetch_src(root_dir):
    # Regex to find the script path
    SCRIPT_PATH_REGEX = re.compile(r"/assets/build/shared_calculator_desktop-[0-9a-f]+\.js")
    # Regex to find the shared module source
    SHARED_MODULE_REGEX = re.compile(r"__dcg_shared_module_source__\s*=\s*(\"(?:\\.|[^\"\\])*\")")

    output_dir = os.path.join(root_dir, OUTPUT_PREFIX + datetime.now().strftime("%Y-%m-%d-%H%M%S"))
    shared_calculator_output_path = os.path.join(output_dir, "shared_calculator_desktop_raw.js")
    shared_module_output_path = os.path.join(output_dir, "shared_module_raw.js")
    shared_module_pretty_output_path = os.path.join(output_dir, "shared_module_pretty.js")

    session = requests.Session()

    # Fetch the page HTML
    response = session.get(BASE_URL)
    response.raise_for_status()

    # Search for the script path
    match = SCRIPT_PATH_REGEX.search(response.text)
    if not match:
        raise RuntimeError("Failed to find shared_calculator_desktop script in the HTML.")

    script_path = match.group(0)
    script_url = urljoin(BASE_URL, script_path)
    print(f"Found shared_calculator_desktop script: {script_url}")

    # Download the JS file
    js_response = session.get(script_url)
    js_response.raise_for_status()

    # We can be reasonably sure that it's not going to fail now, so create the output directory
    os.makedirs(output_dir, exist_ok=True)

    # Save it locally
    with open(shared_calculator_output_path, "wb") as f:
        f.write(js_response.content)
    print(f"Saved shared_calculator_desktop script to '{shared_calculator_output_path}'.")

    # Search for the script path
    for line in js_response.text.split("\n"):
        match = SHARED_MODULE_REGEX.search(line)
        if match:
            break
    else:
        raise RuntimeError("Failed to find the shared module source in the shared_calculator_desktop script.")

    # Python string literals work almost identically to JS string literals, so we can just eval() to unquote
    shared_module = eval(match.group(1))

    # Save the shared module separately
    with open(shared_module_output_path, "w") as f:
        f.write(shared_module)
    print(f"Saved raw shared module source to '{shared_module_output_path}'.")

    # Beautify the shared module source code
    import jsbeautifier
    shared_module_pretty = jsbeautifier.beautify(shared_module)

    # Save the beautified shared module
    with open(shared_module_pretty_output_path, "w") as f:
        f.write(shared_module_pretty)
    print(f"Saved pretty shared module source to '{shared_module_pretty_output_path}'.")

def clean(root_dir):
    import shutil

    for name in os.listdir(root_dir):
        if not name.startswith(OUTPUT_PREFIX):
            continue
        path = os.path.join(root_dir, name)
        if os.path.isdir(path):
            shutil.rmtree(path)

if __name__ == "__main__":
    import sys
    root_dir = os.path.dirname(__file__)
    if len(sys.argv) > 1 and sys.argv[1] == "clean":
        clean(root_dir)
    else:
        fetch_src(root_dir)

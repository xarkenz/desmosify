import re
from urllib.parse import urljoin
import requests
import os
from datetime import datetime
import sys


def print_start(*args, **kwargs):
    kwargs.setdefault("end", "")
    print(*args, **kwargs)
    kwargs.get("file", sys.stdout).flush()


BASE_URL = "https://www.desmos.com/"
OUTPUT_PREFIX = "fetched_"

def fetch_src(root_dir, product = "calculator"):
    # Regex to find the script path
    SCRIPT_PATH_REGEX = re.compile(r"/assets/build/shared_calculator_desktop-[0-9a-f]+\.js")
    # Regex to find the shared module source
    SHARED_MODULE_REGEX = re.compile(r"__dcg_shared_module_source__\s*=\s*(\"(?:\\.|[^\"\\])*\")")

    output_dir = os.path.join(root_dir, OUTPUT_PREFIX + datetime.now().strftime("%Y-%m-%d-%H%M%S"))
    shared_calculator_output_path = os.path.join(output_dir, "shared_calculator.js")
    shared_module_output_path = os.path.join(output_dir, "shared_module.js")

    session = requests.Session()

    # Fetch the page HTML
    page_url = urljoin(BASE_URL, f"/{product}")
    print_start(f"Fetching HTML at '{page_url}'...")
    response = session.get(page_url)
    response.raise_for_status()
    print(" done.")

    # Search for the script path
    match = SCRIPT_PATH_REGEX.search(response.text)
    if not match:
        raise RuntimeError("Failed to find shared calculator script URL in the HTML.")
    script_url = urljoin(BASE_URL, match.group(0))

    # Download the script file
    print_start(f"Fetching script at '{script_url}'...")
    js_response = session.get(script_url)
    js_response.raise_for_status()
    print(" done.")
    shared_calculator = js_response.text

    # Search for the script path in the shared calculator source
    for line in shared_calculator.split("\n"):
        match = SHARED_MODULE_REGEX.search(line)
        if match:
            # Python string literals work almost identically to JS string literals, so we can just eval() to unquote
            shared_module = eval(match.group(1))
            print("Found the shared module script inside the shared calculator script.")
            break
    else:
        print("ERROR: Failed to find the shared_module in the shared calculator script. The output file will be blank.", file=sys.stderr)
        shared_module = ""

    try:
        # Beautify the shared calculator and shared module source code
        import jsbeautifier
        print_start("Formatting shared calculator script...")
        shared_calculator = jsbeautifier.beautify(shared_calculator)
        print(" done.")
        print_start("Formatting shared module script...")
        shared_module = jsbeautifier.beautify(shared_module)
        print(" done.")
    except ImportError:
        print(f"WARNING: jsbeautifier not found; install with pip to get formatted JS files. The raw files will be saved.", file=sys.stderr)
    except:
        print(f"ERROR: An error occurred while formatting JS. The files will be saved, but one or more may not be formatted.", file=sys.stderr)

    # We can be reasonably sure that it's not going to fail now, so create the output directory
    os.makedirs(output_dir, exist_ok=True)

    # Save the shared calculator source
    print_start(f"Saving shared calculator source to '{shared_calculator_output_path}'...")
    with open(shared_calculator_output_path, "w", encoding="utf8") as f:
        f.write(shared_calculator)
    print(" done.")

    # Save the shared module source
    print_start(f"Saving shared module source to '{shared_module_output_path}'...")
    with open(shared_module_output_path, "w", encoding="utf8") as f:
        f.write(shared_module)
    print(" done.")


def clean(root_dir):
    import shutil

    for name in os.listdir(root_dir):
        if not name.startswith(OUTPUT_PREFIX):
            continue
        path = os.path.join(root_dir, name)
        if os.path.isdir(path):
            shutil.rmtree(path)


if __name__ == "__main__":
    root_dir = os.path.dirname(__file__)
    if len(sys.argv) > 1 and sys.argv[1] == "clean":
        clean(root_dir)
    else:
        fetch_src(root_dir)

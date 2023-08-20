import pathlib
import subprocess
import os
import re
import sys

print("Kestrel Automated Test Suite (KATS)")

def check(title: str, name: str, expected: str):
    result = subprocess.run(["./kestrel", "./tests/"+name], capture_output=True)

    expected = expected.replace("\\n", "\n")
    result = result.stderr.replace(b"\\n", b"\n").decode()
    if expected != result:
        print(f"{title}: ❌")
        print(f"Expected:\n'{expected}'\n\nGot:\n'{result}'")
        return False
    else:
        print(f"{title}: ✔️")
        return True

tests = pathlib.Path("tests/tests.txt").read_text()

print("\n\nTests have been loaded, running...")


print("\n========================================\n")

status = []

for test in filter(lambda x: len(x), tests.split("=-=")):
    lines = list(filter(lambda x: len(x), test.splitlines()))
    title = lines[0]
    name = lines[1]
    expected = "\n".join(map(lambda x: x.rstrip(), lines[2:])).strip()+"\n"

    status.append(check(title, name, expected))

    print("\n========================================\n")

print("\n\nRunning doc tests...\n")

for file in os.listdir("docs"):
    data = pathlib.Path(f"docs/{file}").read_text()

    pattern = r'^```(?:\w+)?\s*\n(.*?)(?=^```)```'
    for i, code in enumerate(re.findall(pattern, data, re.DOTALL | re.MULTILINE)):
        with open("./tests/tmp.ke", "w") as f:
            f.write(code)

        status.append(check(f"Code snippet #{i} in {file}", "tmp.ke", ""))

        print("\n========================================\n")


if False in status:
    sys.exit(1)
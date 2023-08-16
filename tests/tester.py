import pathlib
import subprocess
import os
import re

def check(title: str, name: str, expected: str):
    result = subprocess.run(["./kestrel", "./tests/"+name], capture_output=True)

    expected = expected.replace("\\n", "\n")
    result = result.stderr.replace(b"\\n", b"\n").decode()
    if expected != result:
        print(f"{title}: ❌")
        print(f"Expected:\n'{expected}'\n\nGot:\n'{result}'")
    else:
        print(f"{title}: ✔️")

tests = pathlib.Path("tests/tests.txt").read_text()

for test in filter(lambda x: len(x), tests.split("=-=")):
    lines = list(filter(lambda x: len(x), test.splitlines()))
    title = lines[0]
    name = lines[1]
    expected = "\n".join(map(lambda x: x.rstrip(), lines[2:])).strip()+"\n"

    check(title, name, expected)   

for file in os.listdir("docs"):
    data = pathlib.Path(f"docs/{file}").read_text()

    pattern = r'^```(?:\w+)?\s*\n(.*?)(?=^```)```'
    for i, code in enumerate(re.findall(pattern, data, re.DOTALL | re.MULTILINE)):
        with open("./tests/tmp.ke", "w") as f:
            f.write(code)
        check(f"Code snippet #{i} in {file}", "tmp.ke", "")
import sys
import subprocess

"""
.git/hooks/commit-msg

#!/bin/bash

msg=$(cat .git/COMMIT_EDITMSG)

if sh ./hooks/precommit.sh $msg; then
	exit 0
else
	exit 1
fi
"""

arg = " ".join(sys.argv[1:])
if "[ci-skip]" in arg:
    sys.exit(2)

grep_pos = subprocess.run(["grep", "-r", "todo!()", "src"], capture_output=True)
grep_neg = subprocess.run(["grep", "-r", "//todo!()", "src"], capture_output=True)
if len(grep_pos.stdout) != 0 and len(grep_neg.stdout) == 0:
	print(f"todo!() detected:\n{grep_pos.stdout.decode()}")
	sys.exit(1)
    
sys.exit(0)
import sys

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
sys.exit(0)
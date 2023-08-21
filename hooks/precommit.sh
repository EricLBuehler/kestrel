#!/bin/bash

# An example hook script to verify what is about to be committed.
# Called by "git commit" with no arguments.  The hook should
# exit with non-zero status after issuing an appropriate message if
# it wants to stop the commit.

cur=$(pwd)
mkdir /tmp/kestel_precommit 2> /dev/null
cp . /tmp/kestrel_precommit -r 2> /dev/null
cd /tmp/kestrel_precommit

echo "Checking for typos..."
if typos; then
	echo "    No typos found: ✔️"
else
	echo "Typos failed: ❌"
	exit 1
fi

echo "Running KATS..."
res=$(python3 tests/kats.py)
return=$(echo $?)
if ((return==0)); then
	echo "    KATS passed: ✔️"
else
	echo "KATS failed: ❌"
	exit 1
fi

echo "Running precommit script..."
if python3 hooks/precommit.py; then
	echo "    Precommit passed: ✔️"
else
	echo "Precommit failed: ❌"
	exit 1
fi

cd $cur
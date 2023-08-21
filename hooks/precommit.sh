#!/bin/bash

arg="'$*'"

cur=$(pwd)
mkdir /tmp/kestel_precommit 2> /dev/null
cp . /tmp/kestrel_precommit -r 2> /dev/null
cd /tmp/kestrel_precommit

echo "Running precommit script..."
python3 hooks/precommit.py $arg
return=$(echo $?)
if [ $return -eq 0 ]; then
	echo "    Precommit passed: ✔️"

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
    if (return==0); then
        echo "    KATS passed: ✔️"
    else
        echo "KATS failed: ❌"
        exit 1
    fi

elif [ $return -eq 2 ]; then
	echo "    Precommit passed (ci-skip): ✔️"
else
	echo "Precommit failed: ❌"
	exit 1
fi

cd $cur
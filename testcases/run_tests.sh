#!/bin/bash

status=0

echo "Running the Ohua test suite."

# run all tests
for d in */ ; do
    cd $d
    if [ ! -f .skipfile ]; then
        echo "Running test case $d"
        cargo run -q 1>/dev/null
        if [ $? -ne 0 ]; then
            status=1
        fi
    fi
    cd ../
done

# check if error occured
if [ $status -ne 0 ]
then
    echo "An error occured. not all tests exited successfully."
else
    echo "All tests completed successfully."
fi

exit $status

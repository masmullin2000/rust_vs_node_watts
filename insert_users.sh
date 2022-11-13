#!/bin/bash

for i in {1001..2001}
do
    echo "INSERT INTO users (uid, fname, lname) values ($i, 'First_name', 'Last_name');"
done

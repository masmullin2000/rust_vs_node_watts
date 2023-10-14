#!/bin/bash

AMT=$(($1 + 0))

echo "DROP TABLE users;" > insert.sql

echo "CREATE TABLE users (uid INT, fname VARCHAR(255), lname VARCHAR(255));" >> insert.sql

echo "DELETE from users where uid >= 0;" >> insert.sql

for (( i=1; i<=$AMT; i++ )) 
do
    echo "INSERT INTO users (uid, fname, lname) values ($i, 'First_name', 'Last_name');" >> insert.sql
done

sudo mv insert.sql /
sudo -u postgres psql -d list_of_users -a -f /insert.sql
sudo rm -f /insert.sql

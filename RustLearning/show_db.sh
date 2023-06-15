#!/bin/bash

sudo -u postgres psql -d type_record -c "SELECT * FROM users;"

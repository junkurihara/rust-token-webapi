#!/usr/bin/env bash

LOG_FILE=/var/log/token-server/token-server.log
LOG_SIZE=10M
LOG_NUM=10

# logrotate
if [ $LOGROTATE_NUM ]; then
  LOG_NUM=${LOGROTATE_NUM}
fi
if [ $LOGROTATE_SIZE ]; then
  LOG_SIZE=${LOGROTATE_SIZE}
fi

cat > /etc/logrotate.conf << EOF
# see "man logrotate" for details
# rotate log files weekly
weekly
# use the adm group by default, since this is the owning group
# of /var/log/syslog.
su root adm
# keep 4 weeks worth of backlogs
rotate 4
# create new (empty) log files after rotating old ones
create
# use date as a suffix of the rotated file
#dateext
# uncomment this if you want your log files compressed
#compress
# packages drop log rotation information into this directory
include /etc/logrotate.d
# system-specific logs may be also be configured here.
EOF

cat > /etc/logrotate.d/token-server << EOF
${LOG_FILE} {
    dateext
    daily
    missingok
    rotate ${LOG_NUM}
    notifempty
    compress
    delaycompress
    dateformat -%Y-%m-%d-%s
    size ${LOG_SIZE}
    copytruncate
}
EOF

# Check gosu or su-exec, determine linux distribution, and set up user
if [ $(command -v gosu) ]; then
  # Ubuntu Linux
  alias gosu='gosu'
  LINUX="Ubuntu"
  # Setup cron
  mkdir -p /etc/cron.15min/
  cp -p /etc/cron.daily/logrotate /etc/cron.15min/
  echo "*/15 * * * * root cd / && run-parts --report /etc/cron.15min" >> /etc/crontab
  service cron start
elif [ $(command -v su-exec) ]; then
  # Alpine Linux
  alias gosu='su-exec'
  LINUX="Alpine"
  # Setup cron
  cp -f /etc/periodic/daily/logrotate /etc/periodic/15min
  crond -b -l 8
else
  echo "Unknown distribution!"
  exit 1
fi


echo "Start ID Token Server"

# read custom configuration
source /opt/token-server/etc/.env

if [ -z $LOG_LEVEL ]; then
  LOG_LEVEL=info
fi
echo "Log level is ${LOG_LEVEL}"

echo "Run the server"
if [ -n $ADMIN_PASSWORD ]; then
  ADMIN_PASSWORD=${ADMIN_PASSWORD} \
  RUST_LOG=${LOG_LEVEL} \
  /opt/token-server/sbin/rust-token-server run \
  --signing-key-path /opt/token-server/etc/private_key.pem \
  --db-file-path /opt/token-server/var/userdb.db \
  --token-issuer ${TOKEN_ISSUER} \
  --client-ids ${CLIENT_IDS} \
  --listen-address 0.0.0.0 \
  --port=8000
else
  RUST_LOG=${LOG_LEVEL} \
  /opt/token-server/sbin/rust-token-server run \
  --signing-key-path /opt/token-server/etc/private_key.pem \
  --db-file-path /opt/token-server/var/userdb.db \
  --token-issuer ${TOKEN_ISSUER} \
  --client-ids ${CLIENT_IDS} \
  --listen-address 0.0.0.0 \
  --port=8000
fi

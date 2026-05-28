#!/bin/zsh

case "$1" in
  start)
    echo "starting service"
    start_service
    ;;
  stop)
    echo "stopping service"
    stop_service
    ;;
  restart)
    echo "restarting"
    stop_service
    start_service
    ;;
  status)
    check_status
    ;;
  *)
    echo "Usage: $0 {start|stop|restart|status}"
    exit 1
    ;;
esac

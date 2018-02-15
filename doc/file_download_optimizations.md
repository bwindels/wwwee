How to further optimize file downloads (now topping at 3-4mb/s on wifi in local lan with rpi server)

 - experiment with different file reader buffer sizes on download speed
 - analyze tcp traffic with wireshark for bottlenecks (waiting for reply? TCP_CORK? packet size?)
 - see system load and time spent in kernel in epoll_wait and io_submit syscalls with strace

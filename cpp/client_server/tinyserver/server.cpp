#include <fcntl.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <unistd.h>
#include <event2/event.h>
#include <event2/thread.h>

#include "common.h"
#include "server.h"

Stage *Server::session_stage_ = nullptr;

ServerParam::ServerParam() {
  listen_addr_ = INADDR_ANY;
  port_ = PORT_DEFAULT;
  max_connection_ = MAX_CONNECTION_NUM_DEFAULT;
}

void Server::init() {
  session_stage_ = (Stage*)new SessionStage();
}

Server::Server(ServerParam param): server_param_(param) {
  started_ = false;
  server_socket_ = 0;
}

void Server::start() {
  // TODO: add some check
  start_tcp_server();
}

// it should return error code to do error handling
// but we just panic here :)
void Server::start_tcp_server() {
  // create socket
  server_socket_ = socket(AF_INET, SOCK_STREAM, 0);
  if (server_socket_ < 0) {
	common::panic("fail to creat socket");
  }

  // set reuse addr
  int yes = 1;
  if (int ret = setsockopt(server_socket_, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes)); ret < 0) {
	common::panic("fail to set addr reuse");
  }

  // set non block
  set_non_block(server_socket_);

  // bind to addr
  struct sockaddr_in sa;
  memset(&sa, 0, sizeof(sa));
  sa.sin_family = AF_INET;
  sa.sin_port = htons(server_param_.port_);
  sa.sin_addr.s_addr = htonl(server_param_.listen_addr_);

  if (int ret = bind(server_socket_, (struct sockaddr *)&sa, sizeof(sa)); ret < 0) {
	common::panic("fail to bind to addr:port");
  }

  // listen connection
  if (int ret = listen(server_socket_, server_param_.max_connection_)) {
	common::panic("fail to listen");
  }

  // create listen event
  listen_ev_ = event_new(event_base_, server_socket_, EV_READ | EV_PERSIST, accept, this);
  if (listen_ev_ == nullptr) {
	common::panic("fail to create listen_ev_");
  }
  // add listen event to event loop
  if (int ret = event_add(listen_ev_, nullptr); ret < 0) {
	common::panic("fail to add listen_ev_ to event loop");
  }
  started_ = true;
}

void Server::accept(int fd, short ev, void *arg) {
  struct sockaddr_in addr;
  socklen_t addrlen = sizeof(addr);
  // get `this` object
  Server* instance = (Server*)arg;
  // accept new connection
  int client_fd = ::accept(fd, (struct sockaddr *)&addr, &addrlen);
  if (client_fd < 0) {
	common::panic("fail to accept new connection");
  }

  instance->set_non_block(client_fd);

  // create connection object
  Connection* client_connection = new Connection(client_fd);
  // bind client fd to read event which recvive request from client
  event_set(&client_connection->read_event_, client_connection->client_fd_, EV_READ | EV_PERSIST, recv, client_connection);

  // add recv event to even base set
  if (int ret = event_base_set(instance->event_base_, &client_connection->read_event_); ret < 0) {
	common::panic("fail to add read event to event base set");
  }
  // add recv event to main loop
  if (int ret = event_add(&client_connection->read_event_, nullptr); ret < 0) {
	common::panic("fail to add read event to main loop");
  }
}

void Server::recv(int fd, short ev, void *arg) {
  Connection* client_connect = (Connection*)arg;
  int buf_size = sizeof(client_connect->buf);
  memset(&client_connect->buf, 0, buf_size);

  // total data len receive
  int data_len = 0;
  // data len in one read
  int read_len = 0;

  // recvive data slice until meet '\0', EOF or buffer overflow
  while (true) {
	read_len = ::read(fd, client_connect->buf + data_len, buf_size - data_len);
	if (read_len < 0) {
	  if (errno == EAGAIN) {
		continue;
	  }
	  common::panic("fail to read from socket of {}", fd);
	}

	if (read_len == 0) {
	  // finish reading
	  break;
	}

	if (read_len + data_len > buf_size) {
	  // record info and jump to error handling
	  // we just panic here
	  data_len += read_len;
	  common::panic("received data({}) overflow buffer size {}", data_len, buf_size);
	}

	// if meet '\0' we can break;
	bool msg_end = false;
	for (int i = 0; i < read_len; i++) {
	  if (client_connect->buf[i] == 0) {
		msg_end = true;
		data_len = i + 1;
		break;
	  }
	}
	if (msg_end) {
	  break;
	}

	data_len += read_len;
  }

  if (read_len == 0) {
	close_connection(client_connect);
	return;
  }

  // for simplify we handle request here but not use asycn event :)
  session_stage_->handle_event(client_connect);
}

void Server::serve() {
  // create base event
  event_base_ = event_base_new();
  if (event_base_ == nullptr) {
	common::panic("fail to create event base.");
  }

  // create network connection
  start();

  // run main loop
  event_base_dispatch(event_base_);

  // TODO: clean up
}

void Server::set_non_block(int fd) {
  // get origin flag
  int flags = fcntl(fd, F_GETFL);
  if (flags == -1) {
	common::panic("Failed to get flags of fd :%d. ", fd);
  }

  // fill up non block option
  flags = fcntl(fd, F_SETFL, flags | O_NONBLOCK);
  if (flags == -1) {
	common::panic("Failed to set non-block flags of fd :%d. ", fd);
  }
}

void Server::close_connection(Connection *conn) {
  std::cout << "connection close" << std::endl;
  event_del(&conn->read_event_);
  delete conn;
}



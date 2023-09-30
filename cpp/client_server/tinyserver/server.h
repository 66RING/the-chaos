#pragma once

#include <iostream>
#include <event.h>

/* Address to accept any incoming messages.  */
#define	INADDR_ANY ((in_addr_t) 0x00000000)
#define MAX_CONNECTION_NUM_DEFAULT 8192
#define PORT_DEFAULT 16880
#define SOCKET_BUFFER_SIZE 8192

struct Connection{
  int client_fd_;
  struct event read_event_;
  char buf[SOCKET_BUFFER_SIZE];
  Connection(int fd): client_fd_(fd) {}
};

class Stage {
public:
  virtual void handle_event(Connection *conn) = 0;
};

class SessionStage: Stage {
  void handle_request(Connection *conn) {
	std::cout << "request: " << conn->buf << std::endl;
  }

  void handle_event(Connection *conn) {
	handle_request(conn);
  }
};

class ServerParam {
public:
  // setup default server param
  ServerParam();
  // copy construct
  ServerParam(const ServerParam &other) = default;
public:
  long listen_addr_;
  int port_;
  int max_connection_;
};

class Server {
public:
  Server(ServerParam param);
  void send();
  // Create global object of server
  static void init();
  // start serving 
  void serve();

private:
  // accept event handler
  static void accept(int fd, short ev, void *arg);
  // recvive request data from client and notify handler to process request
  static void recv(int fd, short ev, void *arg);
  void start();
  void start_tcp_server();
  // set non block helper function
  void set_non_block(int fd);
  static void close_connection(Connection *conn);

private:
  ServerParam server_param_;
  int server_socket_;
  bool started_ {false};

  static Stage* session_stage_;
  struct event_base *event_base_;
  struct event* listen_ev_;
};

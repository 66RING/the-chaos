#include <errno.h>
#include <netdb.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/un.h>
#include <unistd.h>
#include <iostream>

#define PORT_DEFAULT 16880

std::string prompted_read(std::string prompt) {
  std::string buffer;
  std::cout << prompt;
  std::getline(std::cin, buffer);
  return buffer;
}

bool is_exit_command(std::string cmd) {
  return cmd == "q" || cmd == "exit";
}

int init_tcp_sock(const char *server_host, int server_port)
{
  struct hostent *host;
  struct sockaddr_in serv_addr;

  if ((host = gethostbyname(server_host)) == NULL) {
    fprintf(stderr, "gethostbyname failed. errmsg=%d:%s\n", errno, strerror(errno));
    return -1;
  }

  int sockfd;
  if ((sockfd = socket(AF_INET, SOCK_STREAM, 0)) == -1) {
    fprintf(stderr, "create socket error. errmsg=%d:%s\n", errno, strerror(errno));
    return -1;
  }

  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(server_port);
  serv_addr.sin_addr = *((struct in_addr *)host->h_addr);
  bzero(&(serv_addr.sin_zero), 8);

  if (connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(struct sockaddr)) == -1) {
    fprintf(stderr, "Failed to connect. errmsg=%d:%s\n", errno, strerror(errno));
    close(sockfd);
    return -1;
  }
  return sockfd;
}

int main(int argc, char *argv[])
{
  const char *unix_socket_path = nullptr;
  const char *server_host = "127.0.0.1";
  int server_port = PORT_DEFAULT;
  int opt;

  // args parse
  extern char *optarg;
  while ((opt = getopt(argc, argv, "s:h:p:")) > 0) {
    switch (opt) {
      case 's':
        unix_socket_path = optarg;
        break;
      case 'p':
        server_port = atoi(optarg);
        break;
      case 'h':
        server_host = optarg;
        break;
    }
  }

  std::string prompt_str = "> ";

  int sockfd, send_bytes;

  sockfd = init_tcp_sock(server_host, server_port);
  if (sockfd < 0) {
    return 1;
  }

  std::string input_command;
  while ((input_command = prompted_read(prompt_str)) != "") {
    if (is_exit_command(input_command)) {
      break;
    }

    if ((send_bytes = write(sockfd, input_command.data(), input_command.size() + 1)) == -1) {
      fprintf(stderr, "send error: %d:%s \n", errno, strerror(errno));
      exit(1);
    }
  }
  close(sockfd);

  return 0;
}


#include "server.h"

int main() {
  // create default server parameter
  ServerParam param;
  // create server object
  Server server(param);
  // create global object of server
  Server::init();
  // start serving
  server.serve();
}

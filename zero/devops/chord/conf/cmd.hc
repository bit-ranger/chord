{
  action: {
    docker: {
      enable: true,
      address: "127.0.0.1:2375"
    },
    dubbo: {
      enable: true,
      gateway: {
        registry: {
          protocol: "zookeeper",
          address: "zookeeper://127.0.0.1:2181"
        }
      }
    }
  }
}
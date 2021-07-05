package com.bitranger.dubbo.generic.gateway;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class DubboGenericGatewayApplication {

    public static void main(String[] args) {
        SpringApplication.run(DubboGenericGatewayApplication.class, args);
        System.out.println("----dubbo-generic-gateway-started----");
    }

}

package com.bitranger.dubbo.generic.gateway;

import com.alibaba.dubbo.config.ReferenceConfig;
import com.alibaba.dubbo.rpc.service.GenericService;
import com.bitranger.dubbo.generic.gateway.model.Result;
import lombok.Data;
import lombok.extern.slf4j.Slf4j;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * @author zhangbin
 * @date 2021-03-29
 */
@Slf4j
@RestController
@RequestMapping("/api/dubbo/generic")
public class GenericController {

    @PostMapping("invoke")
    public Result<Object> invoke(@RequestBody DubboGeneric dubboGeneric) {

        dubboGeneric.getReference().setGeneric(true);

        GenericService genericService = dubboGeneric.getReference().get();

        Object result = genericService.$invoke(dubboGeneric.getMethod(), dubboGeneric.getArg_types(), dubboGeneric.getArgs());

        return Result.success(result);
    }

    @Data
    public static class DubboGeneric{
        private ReferenceConfig<GenericService> reference;
        private String method;
        private String[] arg_types;
        private Object[] args;

    }

}

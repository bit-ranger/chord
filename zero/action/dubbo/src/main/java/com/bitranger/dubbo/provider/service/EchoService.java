package com.bitranger.dubbo.provider.service;

import com.bitranger.dubbo.provider.model.Result;

/**
 * @author zhangbin
 * @date 2021-07-05
 */
public interface EchoService {

    Result<String> echo(String content);
}

package com.bitranger.dubbo.generic.gateway.model;

/**
 * @author zhangbin
 * @date 2021-06-08
 */

import java.io.Serializable;

public class Result<T> implements Serializable {
    private static final long serialVersionUID = 1L;
    private boolean success = false;
    private String message;
    private T data;
    private String code;

    public static <T> Result<T> error(String code, String message) {
        return new Result<>(message, null, code);
    }

    public static <T> Result<T> success(T data) {
        return new Result<>(true, "success", data, "0");
    }

    public static <T> Result<T> success() {
        return new Result<>(true, "success", null, "0");
    }

    public Result() {
    }

    public Result(String message, T data, String statusCode) {
        this.message = message;
        this.data = data;
        this.code = statusCode;
    }

    public Result(boolean status, String message, T result, String statusCode) {
        this.success = status;
        this.message = message;
        this.data = result;
        this.code = statusCode;
    }

    public boolean isSuccess() {
        return this.success;
    }

    public void setSuccess(boolean success) {
        this.success = success;
    }

    public String getMessage() {
        return this.message;
    }

    public void setMessage(String message) {
        this.message = message;
    }

    public T getData() {
        return this.data;
    }

    public void setData(T data) {
        this.data = data;
    }

    public String getCode() {
        return this.code;
    }

    public void setCode(String code) {
        this.code = code;
    }
}


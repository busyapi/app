# BusyAPI

## What is BusyAPI?

BusyAPI can simulate long API calls in order to help developpers test their API timeout handling or to simulates long running tasks.

## How to use?

Just call `https://api.busyapi.dev/<timeout>`, where `<timeout>` is the timeout after which the API must response.

If you don't specify any timeout, the API returns immediately.

## Restrictions

The following restrictions applies:

- Maximum timeout is 60s by default, unless changed when runnning the server. If you specify a longer timeout when calling the API, the maximum timeout will be used
- Only `GET` requests are allowed, any other method will result in a `400 Bad Request` error
- Malformed URLs will also result in a `400 Bad Request` error
- Everything after the HTTP method, i.e. request headers, body, etc. will be ignored
- The API always returns a `204 No Content` response when successful

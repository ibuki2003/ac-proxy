# ac-proxy

AtCoder API Proxy to avoid CORS problem

## features

Proxy request, and cache response.

cache data will be purged after every contest **manually**
(Tell me if no purge occurs / Please contribute for better implementation)


## usage

replace host in URL like this:

```diff
-https://atcoder.jp/users/ibuki2003/history/json
+https://ac.fuwa.dev/users/ibuki2003/history/json
```

## supported pages

- `/users/(user)/history/json`
- `/users/(user)/history/json?contestType=heuristic`

## contact

[Issues](https://github.com/ibuki2003/ac-proxy/issues)

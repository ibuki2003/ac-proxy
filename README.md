# ac-proxy

AtCoder API Proxy to avoid CORS problem

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

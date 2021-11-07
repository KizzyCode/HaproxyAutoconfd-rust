[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

# `haproxy_autconfd`
Welcome to `haproxy_autconfd` 🎉

`haproxy_autconfd` is a daemon that automatically assembles a HAProxy config and restarts HAProxy if the config changes.
This container is useful if you use HAProxy as reverse proxy for dynamic container workloads. Backend containers simply
need to write their config fragments into the container-mapped `/usr/local/etc/haproxy.inbox` directory.
`haproxy_autconfd` detects this change, rebuilds the config and restarts HAProxy so that the config is applied
on-the-fly.

## Example
1. Start the container using `docker-compose up`
2. Write the config fragments into the container mapped `/usr/local/etc/haproxy.inbox`:
   1. Create the backend config:
      ```sh
      cat > /path/to/haproxy.inbox/200-mybackend.cfg <<EOF
      backend mybackend
      mode http
      server mybackend backendaddress:port
      http-request set-header X-Forwarded-Port %[dst_port]
      http-request add-header X-Forwarded-Proto https if {{ ssl_fc }}
      EOF
      ```
   2. Write the frontend config:
      ```sh
      cat > /path/to/haproxy.inbox/100-mybackend.cfg <<EOF
      use_backend mybackend if {{ ssl_fc_sni_end -i domainmanagedbybackend }}
      EOF
      ```
3. Delete the config fragments if your backend is removed

## See also
[https://github.com/KizzyCode/HaproxyAutoconfd-rust] for a shadow container that manages backend (de-)registration
automatically.
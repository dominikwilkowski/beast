```
╔╗  ╔═╗ ╔═╗ ╔═╗ ╔╦╗      ╦ ╦ ╦ ╔═╗ ╦ ╦ ╔═╗ ╔═╗ ╔═╗ ╦═╗ ╔═╗
╠╩╗ ║╣  ╠═╣ ╚═╗  ║       ╠═╣ ║ ║ ╦ ╠═╣ ╚═╗ ║   ║ ║ ╠╦╝ ║╣
╚═╝ ╚═╝ ╩ ╩ ╚═╝  ╩       ╩ ╩ ╩ ╚═╝ ╩ ╩ ╚═╝ ╚═╝ ╚═╝ ╩╚═ ╚═╝
            server running on 127.0.0.1:6666
```

# Beast Highscore Server

> This is a REST server to store highscores for Beast

The server is sitting behind an nginx reverse proxy within a `server` block:
```nginx
# Proxy for beast highscore server
#
location /beast/ {
		if ($request_method = 'OPTIONS') {
			add_header 'Access-Control-Allow-Origin' '*';
			add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
			add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range';
			add_header 'Access-Control-Max-Age' 1728000;
			add_header 'Content-Type' 'text/plain; charset=utf-8';
			add_header 'Content-Length' 0;
			return 204;
		}
		if ($request_method = 'POST') {
			add_header 'Access-Control-Allow-Origin' '*';
			add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
			add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range';
			add_header 'Access-Control-Expose-Headers' 'Content-Length,Content-Range';
		}
		if ($request_method = 'GET') {
			add_header 'Access-Control-Allow-Origin' '*';
			add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
			add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range';
			add_header 'Access-Control-Expose-Headers' 'Content-Length,Content-Range';
		}

		proxy_redirect          off;
		proxy_pass_header       Server;
		proxy_set_header        X-Real-IP $remote_addr;
		proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
		proxy_set_header        X-Scheme $scheme;
		proxy_set_header        Host $http_host;
		proxy_set_header        X-NginX-Proxy true;
		proxy_connect_timeout   5;
		proxy_read_timeout      240;
		proxy_intercept_errors  on;

		proxy_pass              http://127.0.0.1:6666/;
}
```

## Deploy the server

Sync files with server:
```sh
make deploy
```

Compile for production on the server:
```sh
cd /www/beast
cargo build --release
```

Install the systemd service:
```sh
sudo cp beast.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable beast.service # for the first time
sudo systemctl start beast.service
sudo systemctl restart beast.service # or restart
sudo systemctl status beast.service
```

Check logs:
```sh
sudo journalctl -u beast.service -r
```

# Reverse Proxy

Plinth binds to localhost by default. Use a reverse proxy to handle TLS and expose it to the internet.

## Caddy (recommended)

Caddy automatically provisions TLS certificates via Let's Encrypt.

```nix
services.caddy = {
  enable = true;
  virtualHosts."example.com".extraConfig = ''
    reverse_proxy localhost:3000
  '';
};

networking.firewall.allowedTCPPorts = [ 80 443 ];
```

## Nginx

```nix
services.nginx = {
  enable = true;
  virtualHosts."example.com" = {
    forceSSL = true;
    enableACME = true;
    locations."/" = {
      proxyPass = "http://127.0.0.1:3000";
      proxyWebsockets = true;
    };
  };
};

security.acme = {
  acceptTerms = true;
  defaults.email = "admin@example.com";
};

networking.firewall.allowedTCPPorts = [ 80 443 ];
```

## Notes

- Plinth serves static assets (WASM, CSS, JS) directly — no separate static file hosting needed
- The image proxy endpoint (`/api/images/`) streams from Immich with 1-year cache headers, so the reverse proxy benefits from caching
- WebSocket support is not currently needed (Leptos uses standard HTTP for SSR)

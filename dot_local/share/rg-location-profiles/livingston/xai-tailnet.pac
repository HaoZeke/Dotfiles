function FindProxyForURL(url, host) {
    if (host === "teachx.ai" || dnsDomainIs(host, ".teachx.ai")) {
        return "SOCKS5 127.0.0.1:1056";
    }
    return "DIRECT";
}

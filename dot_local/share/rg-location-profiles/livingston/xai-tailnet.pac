function FindProxyForURL(url, host) {
    if (host === "starfleet.teachx.ai" ||
        host === "starfleet-backend.teachx.ai" ||
        host === "bifrost.teachx.ai" ||
        host === "nova.teachx.ai" ||
        host === "artifacts-evals.teachx.ai") {
        return "SOCKS5 127.0.0.1:1056";
    }
    return "DIRECT";
}

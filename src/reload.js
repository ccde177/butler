(() => {
    console.log("[butler] script is loaded");
    let url = 'ws://' + window.location.host +'/_butler/ws';
    let ws = new WebSocket(url);

    ws.onmessage = (event) => {
        console.log("[butler] reloaded")
        window.location.reload();
    }

    ws.onclose = () => {
        console.log("[butler] ws connection closed");
    }
})()

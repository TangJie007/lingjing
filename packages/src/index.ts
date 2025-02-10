export default ()=>{
    let Ip = ''
    const RTCPeerConnection = window.RTCPeerConnection || (window as any).mozRTCPeerConnection
    || (window as any).webkitRTCPeerConnection;
    return new Promise((resolve:(v:string)=>void,reject)=>{
        if(RTCPeerConnection){
            const pc = new RTCPeerConnection({
                iceServers:[{urls: "stun:stun.l.google.com:19302"}],
            });
            pc.createDataChannel("");
            pc.createOffer().then((sdp) => {
                pc.setLocalDescription(sdp)
            }).catch((reason) => {
                throw new Error(reason)
            });
            pc.onicecandidate = (ice) => {
                if(ice.candidate){
                    const ip_regex = /([0-9]{1,3}(\.[0-9]{1,3}){3})/
                    const ip_addr = ip_regex.exec(ice.candidate.candidate);
                    if(ip_addr && ip_addr.length !==0){
                        Ip = ip_addr[0]
                        resolve(ip_addr[0])
                    }
                }
            }
    
            const time = setTimeout(()=>{
                if(!Ip){
                    clearTimeout(time)
                    reject(new Error('获取超时'))
                }
            },5000)
        }else{
            reject(new Error('浏览器版本过低，暂不支持'))
        }
    })
}
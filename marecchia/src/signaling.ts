const P2P_API_URL = 'http://localhost:3000';

export class SignalingServer {
    private peerId: string;
    private remote: WebSocket;
    constructor(peerId: string) {
        this.peerId = peerId;
        this.remote = new WebSocket('ws://localhost:3000');
    }
    public async disconnect() {
        const leaveMessage: LeaveMessage = {
            type: SignalingMessageType.LEAVE,
            peerId: this.peerId
        }
        this.remote.send(JSON.stringify(leaveMessage));
    }
}

export enum SignalingMessageType {
    LEAVE = 'leave'
}

//---- Message types ----
interface SignalingMessage {
    type: SignalingMessageType;
    peerId: string;
}

interface LeaveMessage extends SignalingMessage {
    type: SignalingMessageType.LEAVE;
};

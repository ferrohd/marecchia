import { LRUMap } from "../fragments_map";

export type PeerId = string;
export enum PeerError {
    NO_SEGMENT = 'no_segment',
    DISCONNECTED = 'disconnected'
}

export class Peer {
    private peerId: PeerId;
    private rtc: RTCPeerConnection;
    private dataChannel: RTCDataChannel;
    private fragments: LRUMap<ArrayBuffer>;
    private peers: Map<PeerId, Peer>;
    private peerStats: PeerStats;

    constructor(remote_peer: PeerId, peers: Map<PeerId, Peer>, fragments: LRUMap<ArrayBuffer>) {
        this.peerId = remote_peer;
        this.peers = peers;
        this.fragments = fragments;
        this.peerStats = new PeerStats();
        this.rtc = new RTCPeerConnection({
            iceServers: [
                {
                    urls: 'stun:stun.l.google.com:19302'
                }
            ],
        });
        this.dataChannel = this.rtc.createDataChannel('data', {
            maxRetransmits: 3,
            maxPacketLifeTime: 5000,
            ordered: true,
            negotiated: true,
            id: 0,
        });
        this.dataChannel.binaryType = 'arraybuffer';
        // Set the threshold before sending data (64KB)
        this.dataChannel.bufferedAmountLowThreshold = 64 * 1024;
        // Register handler for remote requests
        this.dataChannel.onmessage = this.handle_segment_request;
        // Register handlers for disconnection
        this.dataChannel.addEventListener('close', (_) => {
            this.peers.delete(this.peerId);
        });
        this.dataChannel.addEventListener('error', (_) => {
            this.close();
        });
    }

    private handle_segment_request(event: MessageEvent<any>) {
        const request = JSON.parse(event.data) as RequestSegmentMessage;
        // If the message is not a request, ignore it
        if (request.type !== PeerMessageType.REQUEST_SEGMENT) return;
        // Get the segment and send it back
        const segment = this.fragments.get(request.segmentId);
        const message: ResponseSegmentMessage = {
            type: PeerMessageType.REPONSE_SEGMENT,
            segmentId: request.segmentId,
            data: segment
        };
        this.dataChannel.send(JSON.stringify(message));
        this.peerStats.update_sent();
    }
    /**
     * Request a segment from the peer
     * @param segmentId The ID of the segment to request
     * @returns Promise<ArrayBuffer> The segment data
     * @throws PeerError.NO_SEGMENT if the segment is not available
     * @throws PeerError.DISCONNECTED if the peer is disconnected
     */
    public request_segment(segmentId: string): Promise<ArrayBuffer> {
        return new Promise((resolve, reject) => {
            const message: RequestSegmentMessage = {
                type: PeerMessageType.REQUEST_SEGMENT,
                segmentId
            };
            this.dataChannel.send(JSON.stringify(message));
            // Callback for response
            this.dataChannel.addEventListener('message', (event) => {
                const response = JSON.parse(event.data) as ResponseSegmentMessage;
                if (response.type === PeerMessageType.REPONSE_SEGMENT && response.segmentId === segmentId) {
                    if (response.data) {
                        this.peerStats.update_received();
                        resolve(response.data);
                    } else {
                        reject(PeerError.NO_SEGMENT);
                    }
                }
            });
            // Callbacks
            this.dataChannel.addEventListener('close', (_) => {
                reject(PeerError.DISCONNECTED);
            });
            this.dataChannel.addEventListener('error', (_) => {
                this.close();
                reject(PeerError.DISCONNECTED);
            });
        });
    }

    get id(): PeerId {
        return this.peerId;
    }
    get stats(): PeerStats {
        return this.peerStats;
    }

    public close() {
        this.dataChannel.close();
        this.rtc.close();
    }
}

class PeerStats {
    private fragmentsReceived: number = 0;
    private fragmentsSent: number = 0;
    private lastFragmentExchange: Date | null = null;

    constructor() {
        this.fragmentsReceived = 0;
        this.fragmentsSent = 0;
        this.lastFragmentExchange = null;
    }

    public update_received() {
        this.fragmentsReceived++;
        this.lastFragmentExchange = new Date();
    }

    public update_sent() {
        this.fragmentsSent++;
        this.lastFragmentExchange = new Date();
    }

    public should_disconnect(): boolean {
        // TODO: Implement a proper disconnection policy
        return false
    }

}
// ---- Message types ----
enum PeerMessageType {
    REQUEST_SEGMENT = 'request_segment',
    REPONSE_SEGMENT = 'response_segment'
}
interface PeerMessage {
    type: PeerMessageType;
}
interface RequestSegmentMessage extends PeerMessage {
    type: PeerMessageType.REQUEST_SEGMENT;
    segmentId: string;
}
interface ResponseSegmentMessage extends PeerMessage {
    type: PeerMessageType.REPONSE_SEGMENT;
    segmentId: string;
    data: ArrayBuffer | null;
}

import { LRUMap } from "../fragments_map";
import { SignalingServer } from "../signaling";
import { Peer, PeerId } from "./peer";

export class P2PLoader {
    private fragments: LRUMap<ArrayBuffer>;
    private peerId: PeerId;
    private signalingServer: SignalingServer;
    private peers: Map<PeerId, Peer>;

    constructor(fragmentsMap: LRUMap<ArrayBuffer>) {
        this.fragments = fragmentsMap;
        this.peerId = crypto.randomUUID();
        this.signalingServer = new SignalingServer(this.peerId);
        this.peers = new Map();
    }
    public get_segment(segmentId: string): Promise<ArrayBuffer> {
        // If the segment is already in the fragments map, return it (should never happen)
        const segment = this.fragments.get(segmentId);
        if (segment) return Promise.resolve(segment);

        // If the segment is not in the fragments map, request it from peers
        const requests: Promise<ArrayBuffer>[] = [];
        for (const peer of this.peers.values()) {
            requests.push(peer.request_segment(segmentId));
        }
        return new Promise((resolve, reject) => {
            Promise.any(requests)
            // If any of the requests is successful, return the segment (the first one to resolve)
            .then((segment) => {
                resolve(segment);
            })
            // If all requests fail, reject the promise
            .catch((errors) => {
                reject(errors);
            });
        });
    }
    public disconnect() {
        this.signalingServer.disconnect();
        for (const peer of this.peers.values()) {
            peer.close();
        }
    }
}

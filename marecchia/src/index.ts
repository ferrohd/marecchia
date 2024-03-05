import Hls, { FragmentLoaderContext, HlsConfig, LoadStats, Loader, LoaderCallbacks, LoaderConfiguration, LoaderContext, LoaderStats } from "hls.js";
import { P2PLoader as P2PNetwork } from "./p2p/P2PLoader";
import { LRUMap } from "./fragments_map";

export class P2PFragmentLoader implements Loader<FragmentLoaderContext> {
    private fragments: LRUMap<ArrayBuffer>;
    private p2pNetwork: P2PNetwork;
    private httpLoader: (context: LoaderContext, config: LoaderConfiguration, callbacks: LoaderCallbacks<LoaderContext>) => void;
    context: FragmentLoaderContext | null;
    stats: LoaderStats;

    constructor(confg: HlsConfig) {
        this.fragments = new LRUMap<ArrayBuffer>(10);
        this.p2pNetwork = new P2PNetwork(this.fragments);
        this.httpLoader = new Hls.DefaultConfig.loader(confg).load;
        this.stats = new LoadStats();
        this.context = null;
    }
    load(context: FragmentLoaderContext, config: LoaderConfiguration, callbacks: LoaderCallbacks<FragmentLoaderContext>): void {
        const segmentId = context.frag.sn.toString();

        this.p2pNetwork.get_segment(segmentId)
        .then((segment) => {
            callbacks.onSuccess({
                url: `p2p://${segmentId}`,
                data: segment,
            }, this.stats, context, null);
        })
        .catch((_) => {
            // Try to load the segment from the remote server
            this.httpLoader(context, config, callbacks as LoaderCallbacks<LoaderContext>);
        });
    }
    destroy(): void {
        this.fragments.clear();
        this.p2pNetwork.disconnect();
    }
    abort(): void {
        this.destroy();
    }
}

async function connect_to_peer(remote_offer: RTCSessionDescriptionInit, remotePeerId: string) {
    const config: RTCConfiguration = {
        iceServers: [
            {
                urls: 'stun:stun.l.google.com:19302'
            }
        ]
    };
    const peer = new RTCPeerConnection(config);
    const remoteSessionDescription = new RTCSessionDescription(remote_offer);
    await peer.setRemoteDescription(remoteSessionDescription);
}

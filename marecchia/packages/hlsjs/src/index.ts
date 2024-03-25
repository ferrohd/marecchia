import Hls, { FragmentLoaderConstructor, FragmentLoaderContext, HlsConfig, LoadStats, Loader, LoaderCallbacks, LoaderConfiguration, LoaderContext, LoaderStats } from "hls.js";
import init, { new_p2p_client, P2PClient } from "@marecchia/marecchia-core";

export default init;
export function p2pFragmentLoader(stream_id: string): FragmentLoaderConstructor {
    return class P2PFragmentLoader implements Loader<FragmentLoaderContext> {
        private p2pNetwork: P2PClient;
        private httpLoader: (context: LoaderContext, config: LoaderConfiguration, callbacks: LoaderCallbacks<LoaderContext>) => void;
        context: FragmentLoaderContext | null;
        stats: LoaderStats;

        constructor(confg: HlsConfig) {
            this.p2pNetwork = new_p2p_client(stream_id);
            this.httpLoader = new Hls.default.DefaultConfig.loader(confg).load;
            this.stats = new LoadStats();
            this.context = null;
        }
        load(context: FragmentLoaderContext, config: LoaderConfiguration, callbacks: LoaderCallbacks<FragmentLoaderContext>): void {
            const segmentId = context.frag.sn.toString();

            // P2P exchanges only complete segments (no byte range support)
            context.rangeStart = undefined;
            context.rangeEnd = undefined;

            this.p2pNetwork.request_segment(segmentId)
                .then((segment) => {
                    callbacks.onSuccess({
                        url: `p2p://${segmentId}`,
                        data: segment,
                    }, this.stats, context, null);
                })
                .catch((_) => {
                    // Custom callbacks to upload a new segment once is downloaded from the server
                    const http_callbacks: LoaderCallbacks<FragmentLoaderContext> = {
                        ...callbacks,
                        onSuccess: (response, stats, context, networkDetails) => {
                            if (response.code && response.code === 200 && response.data && response.data instanceof ArrayBuffer) {
                                let data = new Uint8Array(response.data);
                                this.p2pNetwork.send_segment(segmentId, data);
                            }
                            callbacks.onSuccess(response, stats, context, networkDetails);
                        }
                    }
                    // Load the segment using the default HTTP loader
                    this.httpLoader(context, config, http_callbacks as LoaderCallbacks<LoaderContext>);
                });
        }
        destroy(): void {
            this.p2pNetwork.quit();
        }
        abort(): void {
            this.destroy();
        }
    }
}

"use client";
import { useEffect, useRef } from "react";
import Hls, { HlsConfig } from "hls.js";
import init from "@marecchia/marecchia-core";
import { p2pFragmentLoader } from "@marecchia/hlsjs";

export type VideoProps = {
    src: string;
};

export default function Video(props: VideoProps) {
    const videoRef = useRef<HTMLVideoElement>(null);

    useEffect(() => {
        const loadP2P = async () => {
            await init();

            const video = videoRef.current;
            if (!video) return;
            if (!Hls.isSupported()) {
                // Return some error to the user
                return;
            }

            const fLoader = p2pFragmentLoader(props.src);
            console.log(new fLoader(Hls.DefaultConfig));
            const hls = new Hls({
                //debug: process.env.NODE_ENV === "development",
                //fLoader,
                progressive: true,
            });

            hls.targetLatency

            hls.loadSource(props.src);
            hls.attachMedia(video);
        };
        loadP2P();
    }), [props.src, videoRef];

    return (
        <video ref={videoRef} controls />
    );
}

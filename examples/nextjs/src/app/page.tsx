import Video from "@/components/video";
import Image from "next/image";

export default function Home() {
  return (
    <div>
      <h1>Home</h1>
      <Video src="https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8"/>
    </div>
  );
}

import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

type MotionSetup = (motion: typeof gsap) => void;

export function createGsapMotion(setup: MotionSetup) {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return undefined;

  const context = gsap.context(() => setup(gsap));
  return () => context.revert();
}

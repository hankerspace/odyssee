import {
  Trees,
  Rocket,
  Landmark,
  Cpu,
  HeartPulse,
  Palette,
} from 'lucide-vue-next'

export const DOMAIN_CATALOG = [
  {
    id: 'nature',
    icon: Trees,
    sections: [
      { id: 'animals', article: 'arctic-fox' },
      { id: 'oceans', article: 'coral-reef' },
      { id: 'forests', article: 'redwood' },
    ],
  },
  {
    id: 'space',
    icon: Rocket,
    sections: [
      { id: 'planets', article: 'saturn' },
      { id: 'missions', article: 'apollo-11' },
      { id: 'stars', article: 'sun' },
    ],
  },
  {
    id: 'history',
    icon: Landmark,
    sections: [
      { id: 'civilizations', article: 'egypt' },
      { id: 'inventions', article: 'printing-press' },
      { id: 'explorers', article: 'magellan' },
    ],
  },
  {
    id: 'technology',
    icon: Cpu,
    sections: [
      { id: 'robots', article: 'mars-rover' },
      { id: 'energy', article: 'solar-panels' },
      { id: 'transport', article: 'high-speed-train' },
    ],
  },
  {
    id: 'human-body',
    icon: HeartPulse,
    sections: [
      { id: 'skeleton', article: 'femur' },
      { id: 'brain', article: 'neurons' },
      { id: 'senses', article: 'eyes' },
    ],
  },
  {
    id: 'arts',
    icon: Palette,
    sections: [
      { id: 'music', article: 'violin' },
      { id: 'painting', article: 'watercolor' },
      { id: 'sculpture', article: 'marble' },
    ],
  },
]

export const STYLE_WRAPPER = 'A professional educational illustration of [SUBJECT], sticker style, clean lines, bright colors, white background, high quality for children encyclopedia.'

export const SAFETY_GUARD = 'You are a children encyclopedia. Never mention violent, political, or inappropriate content. Stay factual and kind.'

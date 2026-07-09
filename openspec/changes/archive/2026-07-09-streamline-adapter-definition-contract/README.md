# streamline-adapter-definition-contract

收敛 linked adapter 扩展面为单一 registry-facing adapter definition/descriptor：adapter 作者只在 definition 中声明 identity、format metadata、native options、operation handlers 和 full-read grouped capabilities，core/navigation/CLI consumers 从该 definition 传递和派生使用，同时保持现有 document output contract 稳定。

export {};
declare global {
    interface Window {
        electronEvents: Record<string,any>
    }
}
export default class EventManager {
  private readonly events: { [index: string]: ((data: any) => void)[] } = {};
  public on(event: string, callback: (data: any) => void): void {
    if (!this.events[event]) {
      this.events[event] = [];
    }
    this.events[event].push(callback);
  }
  public emit(event: string, data?: any): void {
    const callbacks = this.events[event];
    if (callbacks) {
      for (const callback of callbacks) {
        callback(data);
      }
    }
  }
}

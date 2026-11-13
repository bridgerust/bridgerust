export class DateParseError extends RangeError {
  readonly code: string;

  constructor(code: string) {
    super(code);
    this.name = "DateParseError";
    this.code = code;
  }
}

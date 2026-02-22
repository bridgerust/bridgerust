/* eslint-disable */
/* auto-generated-like local declaration for BridgeTime bindings */

export declare class BridgeDuration {
  constructor(value: bigint | number, unit?: string | null)

  static fromMilliseconds(value: bigint | number): BridgeDuration
  static fromSeconds(value: bigint | number): BridgeDuration
  static fromMinutes(value: bigint | number): BridgeDuration
  static fromHours(value: bigint | number): BridgeDuration
  static fromDays(value: bigint | number): BridgeDuration
  static fromWeeks(value: bigint | number): BridgeDuration
  static fromMonths(value: bigint | number): BridgeDuration
  static fromYears(value: bigint | number): BridgeDuration

  asMilliseconds(): bigint
  asSeconds(): number
  asMinutes(): number
  asHours(): number
  asDays(): number
  asWeeks(): number
  asMonths(): number
  asYears(): number
  humanize(withSuffix?: boolean | null): string
  add(other: BridgeDuration): BridgeDuration
  subtract(other: BridgeDuration): BridgeDuration
  negate(): BridgeDuration
  abs(): BridgeDuration
}

export declare class BridgeTime {
  constructor(input?: string | null, timezone?: string | null)

  static now(timezone?: string | null): BridgeTime
  static parse(input: string, timezone?: string | null): BridgeTime
  static parseFormat(input: string, pattern: string, timezone?: string | null): BridgeTime
  static fromArray(components: Array<bigint | number>, timezone?: string | null): BridgeTime
  static fromUnixMs(unixMs: bigint | number, timezone?: string | null): BridgeTime
  static fromUnix(unixSeconds: bigint | number, timezone?: string | null): BridgeTime
  static duration(value: bigint | number, unit?: string | null): BridgeDuration
  static min(first: BridgeTime, second: BridgeTime): BridgeTime
  static max(first: BridgeTime, second: BridgeTime): BridgeTime

  toIso(): string
  format(pattern: string): string
  unixMs(): bigint
  unix(): bigint
  valueOf(): bigint
  timezone(): string
  toArray(): Array<bigint>
  utcOffset(): number
  isUtc(): boolean
  isDst(): boolean
  toTimezone(timezone: string): BridgeTime
  add(amount: bigint | number, unit: string): BridgeTime
  addDuration(duration: BridgeDuration): BridgeTime
  subtract(amount: bigint | number, unit: string): BridgeTime
  subtractDuration(duration: BridgeDuration): BridgeTime
  startOf(unit: string): BridgeTime
  endOf(unit: string): BridgeTime
  diff(other: BridgeTime, unit: string, asFloat?: boolean | null): number
  isValid(): boolean
  daysInMonth(): number
  isLeapYear(): boolean
  get(field: string): bigint
  set(field: string, value: bigint | number): BridgeTime
  year(): bigint
  month(): bigint
  date(): bigint
  day(): bigint
  hour(): bigint
  minute(): bigint
  second(): bigint
  millisecond(): bigint
  setYear(value: bigint | number): BridgeTime
  setMonth(value: bigint | number): BridgeTime
  setDate(value: bigint | number): BridgeTime
  setDay(value: bigint | number): BridgeTime
  setHour(value: bigint | number): BridgeTime
  setMinute(value: bigint | number): BridgeTime
  setSecond(value: bigint | number): BridgeTime
  setMillisecond(value: bigint | number): BridgeTime
  dayOfYear(): number
  setDayOfYear(value: bigint | number): BridgeTime
  quarter(): number
  setQuarter(value: bigint | number): BridgeTime
  isoWeekday(): number
  setIsoWeekday(value: bigint | number): BridgeTime
  isoWeek(): number
  setIsoWeek(value: bigint | number): BridgeTime
  isoWeekYear(): bigint
  weekOfYear(): number
  week(): number
  setWeek(value: bigint | number): BridgeTime
  weeksInYear(): number
  isoWeeksInYear(): number
  daysInYear(): number
  isToday(): boolean
  isYesterday(): boolean
  isTomorrow(): boolean
  fromTime(other: BridgeTime, withoutSuffix?: boolean | null): string
  toTime(other: BridgeTime, withoutSuffix?: boolean | null): string
  fromNow(withoutSuffix?: boolean | null): string
  toNow(withoutSuffix?: boolean | null): string
  clamp(start: BridgeTime, end: BridgeTime): BridgeTime
  isBefore(other: BridgeTime): boolean
  isAfter(other: BridgeTime): boolean
  isSame(other: BridgeTime): boolean
  isBeforeUnit(other: BridgeTime, unit: string): boolean
  isAfterUnit(other: BridgeTime, unit: string): boolean
  isSameUnit(other: BridgeTime, unit: string): boolean
  isSameOrBefore(other: BridgeTime): boolean
  isSameOrAfter(other: BridgeTime): boolean
  isSameOrBeforeUnit(other: BridgeTime, unit: string): boolean
  isSameOrAfterUnit(other: BridgeTime, unit: string): boolean
  isBetween(
    start: BridgeTime,
    end: BridgeTime,
    unit?: string | null,
    inclusivity?: string | null
  ): boolean
  cloneTime(): BridgeTime
}

export declare function supportedUnits(): Array<string>

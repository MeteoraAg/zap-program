import { BN } from "@coral-xyz/anchor";
import babar from "babar";

export class CuBenchmark {
  public numberOfTestcases = new BN(0);
  public totalCU = new BN(0);
  private worstCase: BN;
  private bestCase: BN;

  private step = 10000;
  private distribution: Array<[number, number]> = [...Array(140).keys()].map(
    (i) => [i * this.step, 0]
  );

  constructor(
    public readonly regex = /consumed (\d+) of (\d+) compute units/
  ) {}

  add = (logs: string[]) => {
    if (!logs[logs.length - 1].endsWith("success")) return;
    for (let i = logs.length - 1; i--; i >= 0) {
      const match = logs[i].match(this.regex);
      if (!match) continue;
      this.numberOfTestcases = this.numberOfTestcases.add(new BN(1));
      const cu = new BN(match[1]);
      // Basics
      this.totalCU = this.totalCU.add(cu);
      this.bestCase = !this.bestCase ? cu : BN.min(this.bestCase, cu);
      this.worstCase = !this.worstCase ? cu : BN.max(this.worstCase, cu);
      // Distribution
      const step = 10000;
      const x = cu.div(new BN(step)).toNumber();
      this.distribution[x][1] += 1;
      break;
    }
  };

  avg = () => {
    return this.totalCU.div(this.numberOfTestcases);
  };

  worst = () => {
    return this.worstCase;
  };

  best = () => {
    return this.bestCase;
  };

  report = () => {
    console.log(
      `Over ${this.numberOfTestcases.toString()} successful testcases:`
    );
    console.log(`\r Avg. CU: ${this.avg().toString()}`);
    console.log(`\r Best CU: ${this.best().toString()}`);
    console.log(`\r Worst CU: ${this.worst().toString()}`);
    console.log(`\r CU Distribution:`);
    console.log(`${babar(this.distribution)}`);
  };
}

export interface FunctionDefinition {
  name: string;
  category: "Math & Trig" | "Statistical" | "Financial" | "Logical" | "Text" | "Lookup & Reference" | "Date & Time";
  syntax: string;
  description: string;
  example: string;
}

export const EXCEL_FUNCTIONS: FunctionDefinition[] = [
  // Math & Trig
  { name: "SUM", category: "Math & Trig", syntax: "SUM(number1, [number2], ...)", description: "Adds all the numbers in a range of cells.", example: "=SUM(A1:A10)" },
  { name: "ABS", category: "Math & Trig", syntax: "ABS(number)", description: "Returns the absolute value of a number.", example: "=ABS(-15)" },
  { name: "ROUND", category: "Math & Trig", syntax: "ROUND(number, num_digits)", description: "Rounds a number to a specified number of digits.", example: "=ROUND(123.456, 2)" },
  { name: "CEILING", category: "Math & Trig", syntax: "CEILING(number, significance)", description: "Rounds a number up, to the nearest multiple of significance.", example: "=CEILING(2.5, 1)" },
  { name: "FLOOR", category: "Math & Trig", syntax: "FLOOR(number, significance)", description: "Rounds a number down, toward zero, to the nearest multiple of significance.", example: "=FLOOR(2.5, 1)" },
  { name: "SQRT", category: "Math & Trig", syntax: "SQRT(number)", description: "Returns a positive square root.", example: "=SQRT(16)" },
  { name: "POWER", category: "Math & Trig", syntax: "POWER(number, power)", description: "Returns the result of a number raised to a power.", example: "=POWER(2, 3)" },
  { name: "MOD", category: "Math & Trig", syntax: "MOD(number, divisor)", description: "Returns the remainder after a number is divided by a divisor.", example: "=MOD(10, 3)" },
  { name: "PRODUCT", category: "Math & Trig", syntax: "PRODUCT(number1, [number2], ...)", description: "Multiplies its arguments.", example: "=PRODUCT(A1:A3)" },
  
  // Statistical
  { name: "AVERAGE", category: "Statistical", syntax: "AVERAGE(number1, [number2], ...)", description: "Returns the average (arithmetic mean) of the arguments.", example: "=AVERAGE(A1:A10)" },
  { name: "COUNT", category: "Statistical", syntax: "COUNT(value1, [value2], ...)", description: "Counts how many numbers are in the list of arguments.", example: "=COUNT(A1:A10)" },
  { name: "COUNTA", category: "Statistical", syntax: "COUNTA(value1, [value2], ...)", description: "Counts how many values are in the list of arguments.", example: "=COUNTA(A1:A10)" },
  { name: "COUNTIF", category: "Statistical", syntax: "COUNTIF(range, criteria)", description: "Counts the number of cells within a range that meet the given condition.", example: '=COUNTIF(A1:A10, ">50")' },
  { name: "MAX", category: "Statistical", syntax: "MAX(number1, [number2], ...)", description: "Returns the maximum value in a list of arguments.", example: "=MAX(A1:A10)" },
  { name: "MIN", category: "Statistical", syntax: "MIN(number1, [number2], ...)", description: "Returns the minimum value in a list of arguments.", example: "=MIN(A1:A10)" },
  { name: "MEDIAN", category: "Statistical", syntax: "MEDIAN(number1, [number2], ...)", description: "Returns the median of the given numbers.", example: "=MEDIAN(A1:A10)" },
  { name: "STDEV", category: "Statistical", syntax: "STDEV(number1, [number2], ...)", description: "Estimates standard deviation based on a sample.", example: "=STDEV(A1:A10)" },

  // Logical
  { name: "IF", category: "Logical", syntax: "IF(logical_test, [value_if_true], [value_if_false])", description: "Specifies a logical test to perform.", example: '=IF(A1>100, "High", "Low")' },
  { name: "AND", category: "Logical", syntax: "AND(logical1, [logical2], ...)", description: "Returns TRUE if all its arguments are TRUE.", example: "=AND(A1>0, B1>0)" },
  { name: "OR", category: "Logical", syntax: "OR(logical1, [logical2], ...)", description: "Returns TRUE if any argument is TRUE.", example: "=OR(A1>0, B1>0)" },
  { name: "NOT", category: "Logical", syntax: "NOT(logical)", description: "Reverses the logic of its argument.", example: "=NOT(A1>10)" },
  { name: "IFS", category: "Logical", syntax: "IFS(logical_test1, value_if_true1, ...)", description: "Checks whether one or more conditions are met and returns a value.", example: '=IFS(A1>90, "A", A1>80, "B")' },
  { name: "IFERROR", category: "Logical", syntax: "IFERROR(value, value_if_error)", description: "Returns a value you specify if a formula evaluates to an error.", example: '=IFERROR(A1/B1, 0)' },

  // Text
  { name: "CONCATENATE", category: "Text", syntax: "CONCATENATE(text1, [text2], ...)", description: "Joins several text items into one text item.", example: '=CONCATENATE(A1, " ", B1)' },
  { name: "LEFT", category: "Text", syntax: "LEFT(text, [num_chars])", description: "Returns the leftmost characters from a text value.", example: "=LEFT(A1, 3)" },
  { name: "RIGHT", category: "Text", syntax: "RIGHT(text, [num_chars])", description: "Returns the rightmost characters from a text value.", example: "=RIGHT(A1, 4)" },
  { name: "MID", category: "Text", syntax: "MID(text, start_num, num_chars)", description: "Returns a specific number of characters from a text string starting at position.", example: "=MID(A1, 2, 4)" },
  { name: "LEN", category: "Text", syntax: "LEN(text)", description: "Returns the number of characters in a text string.", example: "=LEN(A1)" },
  { name: "TRIM", category: "Text", syntax: "TRIM(text)", description: "Removes spaces from text.", example: "=TRIM(A1)" },
  { name: "UPPER", category: "Text", syntax: "UPPER(text)", description: "Converts text to uppercase.", example: "=UPPER(A1)" },
  { name: "LOWER", category: "Text", syntax: "LOWER(text)", description: "Converts text to lowercase.", example: "=LOWER(A1)" },

  // Financial
  { name: "PMT", category: "Financial", syntax: "PMT(rate, nper, pv, [fv], [type])", description: "Calculates the payment for a loan based on constant payments and constant interest rate.", example: "=PMT(0.05/12, 60, -20000)" },
  { name: "PV", category: "Financial", syntax: "PV(rate, nper, pmt, [fv], [type])", description: "Returns the present value of an investment.", example: "=PV(0.05/12, 60, -500)" },
  { name: "FV", category: "Financial", syntax: "FV(rate, nper, pmt, [pv], [type])", description: "Returns the future value of an investment.", example: "=FV(0.05/12, 60, -500)" },
  { name: "NPV", category: "Financial", syntax: "NPV(rate, value1, [value2], ...)", description: "Returns the net present value of an investment based on discount rate and cash flows.", example: "=NPV(0.08, A1:A5)" },
  { name: "IRR", category: "Financial", syntax: "IRR(values, [guess])", description: "Returns the internal rate of return for a series of cash flows.", example: "=IRR(A1:A5)" },

  // Lookup & Reference
  { name: "VLOOKUP", category: "Lookup & Reference", syntax: "VLOOKUP(lookup_value, table_array, col_index, [range_lookup])", description: "Looks in the first column of an array and moves horizontally across the row.", example: "=VLOOKUP(A1, B1:D10, 2, FALSE)" },
  { name: "HLOOKUP", category: "Lookup & Reference", syntax: "HLOOKUP(lookup_value, table_array, row_index, [range_lookup])", description: "Looks in the top row of an array and returns the value of the indicated cell.", example: "=HLOOKUP(A1, A2:Z10, 3, FALSE)" },
  { name: "INDEX", category: "Lookup & Reference", syntax: "INDEX(array, row_num, [column_num])", description: "Uses an index to choose a value from a reference or array.", example: "=INDEX(A1:C10, 2, 3)" },
  { name: "MATCH", category: "Lookup & Reference", syntax: "MATCH(lookup_value, lookup_array, [match_type])", description: "Looks up values in a reference or array.", example: "=MATCH(A1, B1:B10, 0)" },
  { name: "XLOOKUP", category: "Lookup & Reference", syntax: "XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found])", description: "Searches a range or array for a match and returns corresponding item.", example: "=XLOOKUP(A1, B1:B10, C1:C10)" },

  // Date & Time
  { name: "TODAY", category: "Date & Time", syntax: "TODAY()", description: "Returns the serial number of today's date.", example: "=TODAY()" },
  { name: "NOW", category: "Date & Time", syntax: "NOW()", description: "Returns the current date and time.", example: "=NOW()" },
  { name: "DATE", category: "Date & Time", syntax: "DATE(year, month, day)", description: "Returns the serial number of a particular date.", example: "=DATE(2026, 10, 29)" },
  { name: "YEAR", category: "Date & Time", syntax: "YEAR(serial_number)", description: "Converts a serial number to a year.", example: "=YEAR(TODAY())" },
  { name: "MONTH", category: "Date & Time", syntax: "MONTH(serial_number)", description: "Converts a serial number to a month.", example: "=MONTH(TODAY())" },
  { name: "DAY", category: "Date & Time", syntax: "DAY(serial_number)", description: "Converts a serial number to a day of the month.", example: "=DAY(TODAY())" },
];

export class FunctionWizardDialog {
  private element: HTMLElement;
  private isVisible = false;
  private selectedFunction: FunctionDefinition = EXCEL_FUNCTIONS[0];
  private onInsertCallback?: (formulaText: string) => void;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "function-wizard-modal";
    this.element.id = "function-wizard-modal";
    this.applyStyles();
    document.body.appendChild(this.element);
    this.render();
  }

  public setOnInsert(cb: (formulaText: string) => void) {
    this.onInsertCallback = cb;
  }

  public show(preselectedCategory?: string) {
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render(preselectedCategory);
  }

  public hide() {
    this.isVisible = false;
    this.element.style.display = "none";
  }

  public isOpen(): boolean {
    return this.isVisible;
  }

  private applyStyles() {
    this.element.style.display = "none";
    this.element.style.position = "fixed";
    this.element.style.top = "0";
    this.element.style.left = "0";
    this.element.style.width = "100vw";
    this.element.style.height = "100vh";
    this.element.style.backgroundColor = "rgba(0, 0, 0, 0.4)";
    this.element.style.zIndex = "2000";
    this.element.style.alignItems = "center";
    this.element.style.justifyContent = "center";
    this.element.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
  }

  private render(categoryFilter?: string) {
    this.element.innerHTML = `
      <div class="fn-wizard-card" style="background: #ffffff; border-radius: 8px; width: 540px; box-shadow: 0 16px 36px rgba(0,0,0,0.25); overflow: hidden; border: 1px solid #d1d5db; display: flex; flex-direction: column;">
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 18px; border-bottom: 1px solid #e5e7eb; background: #f9fafb;">
          <span style="font-weight: 600; font-size: 14px; color: #111827;">Insert Function (ƒx)</span>
          <button class="btn-close-fn-wizard" style="background: none; border: none; font-size: 16px; cursor: pointer; color: #9ca3af;">✕</button>
        </div>
        
        <div style="padding: 16px 20px; display: flex; flex-direction: column; gap: 12px;">
          <div style="display: flex; gap: 8px;">
            <input type="text" class="fn-search-input" placeholder="Search for a function..." style="flex: 1; padding: 6px 10px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 13px;" />
            <select class="fn-category-select" style="padding: 6px 10px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 13px; background: #ffffff;">
              <option value="All">All Categories</option>
              <option value="Math & Trig">Math & Trig</option>
              <option value="Statistical">Statistical</option>
              <option value="Logical">Logical</option>
              <option value="Text">Text</option>
              <option value="Financial">Financial</option>
              <option value="Lookup & Reference">Lookup & Reference</option>
              <option value="Date & Time">Date & Time</option>
            </select>
          </div>

          <div style="display: flex; gap: 12px; height: 200px;">
            <div class="fn-list-container" style="flex: 1; border: 1px solid #d1d5db; border-radius: 4px; overflow-y: auto; background: #ffffff;">
              <!-- Function list items -->
            </div>
            <div class="fn-details-container" style="flex: 1.2; border: 1px solid #e5e7eb; border-radius: 4px; padding: 12px; background: #f9fafb; display: flex; flex-direction: column; gap: 8px; font-size: 12px;">
              <div style="font-weight: 700; font-size: 14px; color: #107c41;" class="fn-detail-name">SUM</div>
              <div style="font-family: monospace; font-size: 11px; color: #1f2937;" class="fn-detail-syntax">SUM(number1, [number2], ...)</div>
              <div style="color: #4b5563; flex: 1;" class="fn-detail-desc">Adds all the numbers in a range of cells.</div>
              <div style="color: #6b7280; font-size: 11px;">Example: <strong class="fn-detail-example">=SUM(A1:A10)</strong></div>
            </div>
          </div>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px; padding: 12px 18px; border-top: 1px solid #e5e7eb; background: #f9fafb;">
          <button class="btn-cancel-fn" style="padding: 6px 14px; border: 1px solid #d1d5db; border-radius: 4px; background: #ffffff; cursor: pointer; font-size: 13px;">Cancel</button>
          <button class="btn-insert-fn" style="padding: 6px 18px; border: 1px solid #107c41; border-radius: 4px; background: #107c41; color: #ffffff; cursor: pointer; font-size: 13px; font-weight: 600;">Insert</button>
        </div>
      </div>
    `;

    const closeBtn = this.element.querySelector<HTMLButtonElement>(".btn-close-fn-wizard")!;
    const cancelBtn = this.element.querySelector<HTMLButtonElement>(".btn-cancel-fn")!;
    const insertBtn = this.element.querySelector<HTMLButtonElement>(".btn-insert-fn")!;
    const searchInput = this.element.querySelector<HTMLInputElement>(".fn-search-input")!;
    const categorySelect = this.element.querySelector<HTMLSelectElement>(".fn-category-select")!;
    const listContainer = this.element.querySelector<HTMLElement>(".fn-list-container")!;

    if (categoryFilter) {
      categorySelect.value = categoryFilter;
    }

    const updateDetails = (fnDef: FunctionDefinition) => {
      this.selectedFunction = fnDef;
      const nameEl = this.element.querySelector(".fn-detail-name")!;
      const syntaxEl = this.element.querySelector(".fn-detail-syntax")!;
      const descEl = this.element.querySelector(".fn-detail-desc")!;
      const exEl = this.element.querySelector(".fn-detail-example")!;
      nameEl.textContent = fnDef.name;
      syntaxEl.textContent = fnDef.syntax;
      descEl.textContent = fnDef.description;
      exEl.textContent = fnDef.example;

      listContainer.querySelectorAll(".fn-item").forEach((item) => {
        item.classList.toggle("active", (item as HTMLElement).dataset.fnName === fnDef.name);
      });
    };

    const populateList = () => {
      listContainer.innerHTML = "";
      const q = searchInput.value.trim().toLowerCase();
      const cat = categorySelect.value;

      const filtered = EXCEL_FUNCTIONS.filter((fn) => {
        const matchesCat = cat === "All" || fn.category.toLowerCase().includes(cat.toLowerCase());
        const matchesQuery = !q || fn.name.toLowerCase().includes(q) || fn.description.toLowerCase().includes(q);
        return matchesCat && matchesQuery;
      });

      if (filtered.length === 0) {
        listContainer.innerHTML = `<div style="padding: 12px; color: #9ca3af; text-align: center; font-size: 12px;">No functions found</div>`;
        return;
      }

      filtered.forEach((fn, idx) => {
        const item = document.createElement("div");
        item.className = `fn-item ${idx === 0 ? "active" : ""}`;
        item.dataset.fnName = fn.name;
        item.textContent = fn.name;
        item.style.padding = "6px 12px";
        item.style.cursor = "pointer";
        item.style.fontSize = "13px";
        item.style.borderBottom = "1px solid #f3f4f6";

        item.onclick = () => updateDetails(fn);
        item.ondblclick = () => {
          updateDetails(fn);
          this.executeInsert();
        };

        listContainer.appendChild(item);
      });

      if (filtered.length > 0) {
        updateDetails(filtered[0]);
      }
    };

    closeBtn.onclick = () => this.hide();
    cancelBtn.onclick = () => this.hide();
    insertBtn.onclick = () => this.executeInsert();

    searchInput.oninput = () => populateList();
    categorySelect.onchange = () => populateList();

    populateList();
    searchInput.focus();
  }

  private executeInsert() {
    this.hide();
    if (this.onInsertCallback) {
      this.onInsertCallback(this.selectedFunction.example);
    }
  }
}


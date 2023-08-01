use std::fmt::Display;

use enum_iterator::{all, Sequence};
use leptos::*;
use leptos_router::*;
use uiua::{primitive::Primitive, IoOp};

use crate::{backend::EXAMPLE_UA, code::*, editor::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
#[repr(u8)]
pub enum TutorialPage {
    Basic,
    Math,
    Arrays,
    Types,
    Bindings,
    Functions,
    Modules,
}

impl TutorialPage {
    pub fn path(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
    pub fn title(&self) -> &'static str {
        match self {
            Self::Basic => "Basic Stack Operations and Formatting",
            Self::Math => "Math and Comparison",
            Self::Arrays => "Arrays",
            Self::Types => "Types",
            Self::Bindings => "Bindings",
            Self::Functions => "Modifiers and Functions",
            Self::Modules => "Modules",
        }
    }
}

#[component]
pub fn Tutorial(cx: Scope, page: TutorialPage) -> impl IntoView {
    let tut_view = match page {
        TutorialPage::Basic => view! { cx, <TutorialBasic/> }.into_view(cx),
        TutorialPage::Math => view! { cx, <TutorialMath/> }.into_view(cx),
        TutorialPage::Arrays => view! { cx, <TutorialArrays/> }.into_view(cx),
        TutorialPage::Types => view! { cx, <TutorialTypes/> }.into_view(cx),
        TutorialPage::Bindings => view! { cx, <TutorialBindings/> }.into_view(cx),
        TutorialPage::Functions => view! { cx, <TutorialFunctions/> }.into_view(cx),
        TutorialPage::Modules => view! { cx, <TutorialModules/> }.into_view(cx),
    };
    view! { cx,
        <TutorialNav page=page/>
        { tut_view }
        <br/>
        <br/>
        <TutorialNav page=page/>
    }
}

impl IntoParam for TutorialPage {
    fn into_param(value: Option<&str>, name: &str) -> Result<Self, ParamsError> {
        all::<TutorialPage>()
            .find(|p| p.path() == value.unwrap_or(""))
            .ok_or_else(|| ParamsError::MissingParam(name.to_string()))
    }
}

#[component]
fn TutorialNav(cx: Scope, page: TutorialPage) -> impl IntoView {
    let next = move || {
        page.next()
            .map(|p| {
                view!(cx, <div><A href=format!("/docs/{}", p.path())>{p.title()}</A>" 〉"</div>)
                    .into_view(cx)
            })
            .unwrap_or_else(|| view!(cx, <div/>).into_view(cx))
    };
    let previous = move || {
        page.previous()
            .map(|p| {
                view!(cx, <div>"〈 "<A href=format!("/docs/{}", p.path())>{p.title()}</A></div>)
                    .into_view(cx)
            })
            .unwrap_or_else(|| view!(cx, <div/>).into_view(cx))
    };

    view! { cx,
        <div class="tutorial-nav">
            { previous }
            { next }
        </div>
    }
}

#[component]
fn TutorialBasic(cx: Scope) -> impl IntoView {
    let primitive_table: Vec<_> = Primitive::all()
        .filter_map(|p| {
            if let (Some(name), Some(ascii), Some(_)) = (p.name(), p.ascii(), p.unicode()) {
                Some(view! { cx,
                    <tr>
                        <td><code>{ name }</code></td>
                        <td><code>{ ascii.to_string() }</code></td>
                        <td><PrimCode prim=p glyph_only=true/></td>
                    </tr>
                })
            } else {
                None
            }
        })
        .collect();

    view! { cx,
        <div>
            <h2>"Basic Stack Operations and Formatting"</h2>
            <h2>"The Stack"</h2>
            <hr/>
            <p>"In Uiua, all operations operate on a global stack. Each line is evaluated from right to left. A number simply pushes its value onto the stack."</p>
            <Editor example="5"/>
            <Editor example="1 2 3"/>
            <p>"Operators pop values off the stack and push their results."</p>
            <p>"For example, "<PrimCode prim=Primitive::Add glyph_only=true/>" pops two values off the stack and pushes their sum."</p>
            <Editor example="+ 1 2"/>
            <p><PrimCode prim=Primitive::Mul glyph_only=true/>", of course, multiplies the two values instead."</p>
            <Editor examples={&["3", "2", "×", "1", "+"]} help={&["", "Try the arrows to see how the stack changes with each operation."]}/>
            <p>"In the editor, items that end up on the "<i>"top"</i>" of the stack are shown at the "<i>"bottom"</i>" of the output. This is so consecutive lines of code show their outputs in the correct order."</p>
            <Editor example="5\n+1 2\n\"Hello, World!\"\n+1 'a'"/>
            <br/>
            <h2>"Formatting"</h2>
            <hr/>
            <p>"Most Uiua built-in functions use special unicode characters. To type multiplication and division signs, you can use "<code>"*"</code>" and "<code>"%"</code>" respectively. Then, run the code to format."</p>
            <Editor example="# Click Run to format!\n%6 *3 8" help={&["", "⇡Click⇡"]}/>
            <p>"Most built-in functions have names you can type rather than symbols. Formatting works on these too. "<b>"This is the primary way of entering Uiua's glyphs."</b></p>
            <Editor example="max sqrt 10 mod 10 pow 2 8"/>
            <Editor example="* `1 `2"/>
            <p>"You don't have to type the whole name, just enough to disambiguate it from others"</p>
            <Editor example="(cei ceil ceili ceilin ceiling)"/>
            <p>"On this site, you can also click the ↧ symbol on any editor to show a pallete of all the Uiua glyphs. You can then click on any glyph to insert it into the editor."</p>
            <p>"Here is a table of all the glyphs that are typed with ASCII characters that get converted to glyphs."</p>
            <table>
                <tr>
                    <th>"Name"</th>
                    <th>"ASCII"</th>
                    <th>"Glyph"</th>
                </tr>
                {primitive_table}
                <tr>
                    <td>"negative number"</td>
                    <td><code>"`"</code></td>
                    <td><code>"¯"</code></td>
                </tr>
            </table>
            <p>"As noted in the table, negative number literals are typed with the "<code>"`"</code>" character. This is because "<code>"-"</code>" is used for subtraction."</p>
            <Editor example="+ `1 `2"/>
        </div>
    }
}

fn maybe_code<T: Display>(cx: Scope, val: Option<T>) -> impl IntoView {
    if let Some(val) = val {
        view! { cx, <code>{ val.to_string() }</code> }.into_view(cx)
    } else {
        view! { cx, "" }.into_view(cx)
    }
}

fn primitive_rows(cx: Scope, prims: impl IntoIterator<Item = Primitive>) -> Vec<impl IntoView> {
    prims
        .into_iter()
        .map(|p| {
            let name = p.name();
            let glyph = p.unicode();
            let ascii = p
                .ascii()
                .map(|s| s.to_string())
                .or_else(|| glyph.filter(|c| c.is_ascii()).map(|c| c.to_string()));
            view! { cx,
                <tr>
                    <td>{maybe_code(cx, name)}</td>
                    <td>{maybe_code(cx, ascii)}</td>
                    <td><PrimCode prim=p glyph_only=true/></td>
                    <td>{view!(cx, <code>{p.args()}</code>)}</td>
                </tr>
            }
        })
        .collect()
}

#[component]
fn TutorialMath(cx: Scope) -> impl IntoView {
    use Primitive::*;
    let math_table = primitive_rows(
        cx,
        [
            Add, Sub, Mul, Div, Mod, Pow, Neg, Abs, Ceil, Floor, Round, Sqrt, Sign,
        ],
    );
    let comp_table = primitive_rows(cx, [Eq, Ne, Lt, Gt, Le, Ge, Min, Max, Floor, Ceil, Round]);

    view! { cx,
        <div>
            <h2>"Math and Comparison"</h2>
            <p>"Uiua supports all the basic math operations as well as comparison, min/max, and rounding."</p>
            <div style="display: flex; justify-content: space-evenly;">
                <table class="bordered-table">
                    <tr>
                        <th>"Name"</th>
                        <th>"ASCII"</th>
                        <th>"Glyph"</th>
                        <th>"Args"</th>
                    </tr>
                    {math_table}
                </table>
                <table class="bordered-table">
                    <tr>
                        <th>"Name"</th>
                        <th>"ASCII"</th>
                        <th>"Glyph"</th>
                        <th>"Args"</th>
                    </tr>
                    {comp_table}
                </table>
            </div>
            <p>"Most of these are used mostly how you might think."</p>
            <Editor example="+2 5"/>
            <Editor example="↥2 5"/>
            <Editor example="ⁿ2 5"/>
            <Editor example="⌈2.5"/>
            <Editor example="√4"/>
            <p>"One thing to note is that non-commutative operators work backwards."</p>
            <p>"This is so you can think of the operator and the second number as a single unit."</p>
            <Editor example="-2 5" help={&["", "What is 5 \"minus 2\"?"]}/>
            <Editor example="<2 5" help={&["", "Is 5 \"less than 2\"?"]}/>
            <Editor example="÷2 5" help={&["", "What is 5 \"divided by 2\"?"]}/>
            <p>"Uiua has no boolean types. Comparison operators return "<code>0</code>" for false and "<code>1</code>" for true."</p>
            <Editor example="=2 5"/>
            <Editor example="=2 2"/>
            <p>"Because of how stack operations work, you can delay operations until after all the arguments are on the stack."</p>
            <Editor examples={&["4", "3", "2", "1", "+", "+", "×"]} mode=EditorMode::Progressive help={&["", "Click the arrows to see how the expression is built up"]}/>
            <p>"This is not special syntax. All the numbers are pushed to the stack, then the operators work on them."</p>
            <p>"Remember that you can type the names of operators and then run to format them."</p>
            <Editor example="# Click Run to format!\nmax sqrt 2 mod 10 abs `31" help={&["", "⇡Click⇡"]}/>
        </div>
    }
}

#[component]
fn TutorialArrays(cx: Scope) -> impl IntoView {
    use Primitive::*;
    view! { cx,
        <div>
            <h2>"Arrays"</h2>
            <p>"Uiua is, first and foremost, an array language. The only composite data type is the multimensional array. Arrays have a lot of nice properties, and the language's built-in functions are designed to make it easy to work with them. If you've only ever programmed in non-array languages, then this will be a completely foreign paradigm. In most array languages, most data structures and control flow are replaced with operations on arrays."</p>
            <h2>"Creating Arrays"</h2>
            <p>"Other than with functions, Uiua has two ways to create arrays. They are called "<i>"strand notation"</i>" and "<i>"stack notation"</i>"."</p>
            <p><b>"Strand notation"</b>" uses underscores to connect elements."</p>
            <Editor example="1_2_3"/>
            <Editor example="\"Hello\"_\"World\""/>
            <Editor example="+_-_×_÷"/>
            <p>"Strand notation is good when you want to create short and/or simple arrays. For longer or more complex arrays, you can use stack notation."</p>
            <p><b>"Stack notation"</b>" uses brackets to group elements."</p>
            <Editor example="[1 2 3]"/>
            <Editor example="[¯5 37 42 π]"/>
            <p>"What's cool about stack notation is that it is "<i>"not"</i>" just a way to list elements. The code between the brackets runs from right to left as it normally would. When it is done, any items on the stack higher than when it started are put into the array. This gives you some cool ways to create arrays."</p>
            <p>"Note that "<PrimCode prim=Dup/>" duplicates the top item on the stack."</p>
            <Editor example="[...5]"/>
            <Editor example="[×2.×2.×2.×2 .2]"/>
            <Editor example="[+1 2 +3 4]"/>
            <p>"Of course, you can also use stack notation to make multidimensional arrays."</p>
            <Editor example="[1_2_3 4_5_6]"/>
            <Editor example="[...[1 2 3]]"/>
            <p>"More preceisely, stack notation "<PrimCode prim=Couple/>"s the first two stack items created between the "<code>"[]"</code>" and "<PrimCode prim=Join/>"s the to that coupling."</p>
            <br/>
            <h2><PrimCode prim=Shape/>", "<PrimCode prim=Len/>", and "<PrimCode prim=Rank/></h2>
            <hr/>
            <p>"Other than their data, arrays also have a property called their "<b>"shape"</b>". Shape is a list of non-negative integers that describes the array's size along each of its axes."</p>
            <p>"We can get the array's shape with the "<PrimCode prim=Shape/>" function. It's a triangle because a triangle is a shape."</p>
            <Editor example="△[1 2 3]"/>
            <Editor example="△5"/>
            <Editor example="△[[1 2 3] [4 5 6]]"/>
            <Editor example="△[...[1 2 3]]"/>
            <p>"From shape we can derive two closely-related properties called "<b>"length"</b>" and "<b>"rank"</b>"."</p>
            <p><PrimCode prim=Len/>" is the number of "<i>"major cells"</i>" of the array. This is the number of elements for a 1D array and the number of rows for a 2D array. Length is always equal to the first number in the shape (or 1 if the shape is empty)."</p>
            <p><PrimCode prim=Rank/>" is the number of dimensions of the array."</p>
            <Editor example="△[1 2 3]\n≢[1 2 3]\n∴[1 2 3]"/>
            <p><PrimCode prim=Rank/>" is equivalent to the "<PrimCode prim=Len/>" of the "<PrimCode prim=Shape/>"."</p>
            <Editor example="= ∴[1 2 3] ≢△[1 2 3]"/>
            <br/>
            <h2>"Pervasion"</h2>
            <hr/>
            <p>"Most operations that apply to scalars are what is called "<i>"pervasive"</i>" when it comes to arrays. This means that the operations automatically applies to every item in the array."</p>
            <Editor example="+1 1_2_3\n√[4 9 16]\n+1_2_3 4_5_6"/>
            <p>"When doing a pervasive operation on two arrays, their shape "<i>"prefixes"</i>" must match."</p>
            <Editor example="+[1 2] [3 4 5]"/>
            <Editor example="△10_20\n△[3_4_5 6_7_8]\n+10_20 [3_4_5 6_7_8]"/>
            <br/>
            <h2>"The Flat Array Model"</h2>
            <hr/>
            <p>"Most modern array languages allow arrays to contain arrays. The simplest model for this is the "<a href="https://aplwiki.com/wiki/Box">"Boxed Array Model"</a>" used in J, where an array may be an array of \"boxes\", which can contain any value but which must be boxed and unboxed. APL uses the "<a href="https://aplwiki.com/wiki/Array_model#Nested_array_theory">"Nested Array Model"</a>", making nested arrays a little easier to access. BQN uses the "<a href="https://mlochbaum.github.io/BQN/doc/based.html">"Based Array Model"</a>"."</p>
            <p>"Uiua does not use any of these. It sticks to the Flat Array Model of the original APL. In Uiua, "<b>"you cannot nest arrays inside other arrays"</b>"."</p>
            <p>"This may seem like a regression, but this model was picked for two reasons. For one, it is easier to make array algorithms fast when you only have to check their type once. More importantly than the implementation, however, is that the Flat Array Model makes array functions easier to reason about. In Uiua, you never have to wonder whether a function like "<PrimCode prim=Windows/>" or "<PrimCode prim=Partition/>" return an array or an array of arrays. The answer is always the same."</p>
            <p>"If you have worked with other array languages, you may be thinking, \"Sure, the Flat Array Model is simple, but it is very limited. How do you represent arrays with subarrays of different lengths?\" The answer is "<b>"fill values"</b>"."</p>
            <p>"Many operations that work on multiple arrays will insert fill values when shapes do not match. The most basic is stack notation, like in the error in the example above."</p>
        </div>
    }
}

#[component]
fn TutorialTypes(cx: Scope) -> impl IntoView {
    use Primitive::*;
    view! { cx,
        <h2>"Types"</h2>
        <p>"Every value in Uiua is an array. However, different arrays on the stack can have different "<em>"types"</em>" of items. Every element of an array is always the same type. Unlike some other array programming languages, Uiua arrays cannot have elements of different types."</p>
        <p>"There are only three types of arrays:"</p>
        <ul>
            <li><b>"Number"</b></li>
            <li><b>"Character"</b></li>
            <li><b>"Function"</b></li>
        </ul>
        <br/>
        <h2>"Numbers"</h2>
        <hr/>
        <p>"Numbers are decimal numbers with floating precision. They are represented as 64-bit floating-point."</p>
        <Editor example="[5 0 3.2 ¯1.1 π ∞]"/>
        <p>"Most math operations can only be applied to numbers."</p>
        <p>"Even though numbers can have a fractional part, many built-in functions require whole numbers. These functions will return an error if given a non-whole number."</p>
        <p>"One such example is "<PrimCode prim=Pick/>"."</p>
        <Editor example="⊡ 2 [4 7 9 1 0]"/>
        <Editor example="⊡ 3.1 [4 7 9 1 0]"/>
        <p>"If you want to convert a number to a whole number, you can use "<PrimCode prim=Floor/>", "<PrimCode prim=Ceil/>", or "<PrimCode prim=Round/>"."</p>
        <br/>
        <h2>"Characters"</h2>
        <hr/>
        <p>"Characters are represented as 32-bit Unicode codepoints."</p>
        <p>"Character literals, delimited by "<code>"'"</code>"s, create "<PrimCode prim=Rank/><code>"0"</code>" character arrays."</p>
        <Editor example="['a' 'b' 'c']"/>
        <p>"String literals, delimited by "<code>"\""</code>"s, create "<PrimCode prim=Rank/><code>"1"</code>" character arrays."</p>
        <Editor example="\"Hello, World!\""/>
        <p>"Characters and numbers exist in an "<a href="https://en.wikipedia.org/wiki/Affine_space">"affine space."</a></p>
        <p>"You can add or subtract a number from a character to get another character."</p>
        <p>"You can subtract two characters to get a number."</p>
        <p>"No other arithmetic operations can be done on characters."</p>
        <Editor example="+1 'a'"/>
        <Editor example="-'a' 'z'"/>
        <Editor example="+'a' 'b'"/>
        <br/>
        <h2>"Functions"</h2>
        <hr/>
        <p>"Functions are usually used as scalars, but they are still arrays. Most array operations that work on number and character arrays work on arrays of functions as well."</p>
        <p>"Functions will be discussed more in a "<A href="/docs/functions">"later section"</A>"."</p>
        <br/>
        <h2>"Type agreement"</h2>
        <hr/>
        <p>"For functions that work on the structure of arrays rather than their elements, the types of the arrays must match."</p>
        <Editor example="⊂ 1_2 3"/>
        <Editor example="⊟ \"Hello\" \"World\""/>
        <Editor example="⊟ 1_2_3 \"dog\""/>
    }
}

#[component]
fn TutorialBindings(cx: Scope) -> impl IntoView {
    use Primitive::*;
    view! { cx,
        <div>
            <h2>"Bindings"</h2>
            <p>"Bindings are global names that can be given to Uiua values. They are denoted with "<code>"←"</code>", which the formatter will convert from "<code>"="</code>" when appropriate."</p>
            <Editor example="a = 3\nb = 5\n+ a b" help={&["", "Try running to format the ="]}/>
            <p>"Valid binding names can be made up of any sequence of uppercase or lowercase alphabetic characters OR a single non-alphanumeric character that is not already used for a Uiua function."</p>
            <p>"Unlike most programming languages, binding names in Uiua "<i>"cannot"</i>" contain numbers or underscores."</p>
            <Editor example="numone ← 1\nNuMtWo ← 2\n😀 ← \"happy\""/>
            <Editor example="variable_1 ← 5"/>
            <p>"Bindings are case-insensitive."</p>
            <Editor example="foo ← 5\n+ FOO fOo"/>
            <p>"The compiler can somtimes mistake all-lowercase binding names for unformatted built-in functions."</p>
            <p>"Here, the compiler thinks that "<code>"part"</code>" is "<PrimCode prim=Partition/>"."</p>
            <Editor example="part = 5" help={&["", "Run to format and reveal why this does not work"]}/>
            <p>"To fix this issue, simply change the binding's capitalization."</p>
            <Editor example="Part ← 5\n×2 Part"/>
            <br/>
            <h2>"Binding Functions"</h2>
            <hr/>
            <p>"If you start a binding with a captial letter or an unused glyph, it will bind as a function."</p>
            <Editor example="TimesThree ← ×3\nTimesThree 7"/>
            <Editor example="👋 ← ⊂\"Hello, \"\n👋 \"World!\""/>
            <p>"The next section discusses functions in more detail."</p>
        </div>
    }
}

#[component]
fn TutorialFunctions(cx: Scope) -> impl IntoView {
    use Primitive::*;
    view! { cx,
        <div>
            <h2>"Modifiers and Functions"</h2>
            <h2>"Modifiers"</h2>
            <hr/>
            <p>"Modifiers are functions that take other functions. If you immediately follow a modifier with its function arguments, the functions will be called inside the modifier rather than outside."</p>
            <p>"For example, "<PrimCode prim=Reduce/>" applies a function \"between\" all rows of an array."</p>
            <Editor example="/+ 1_2_3_4"/>
            <p><PrimCode prim=Scan/>" is similar, but it returns all the intermediate results."</p>
            <Editor example="\\+ 1_2_3_4"/>
            <p><A href="/docs">"The main docs page"</A>" lists all of the built-in modifiers."</p>
            <br/>
            <h2>"Functions"</h2>
            <hr/>
            <p>"In addition to creating a new function with a capitalized binding name, as discussed in the "<A href="/docs/bindings">"previous sections"</A>", functions in Uiua can also be created with "<code>"(...)"</code>"."</p>
            <p>"This is usually only necessary when you need to call multiple functions within a modifier."</p>
            <p>"For example, if you wanted to make an array that is pairs each element of an array with its inverse, you could use "<PrimCode prim=Each/>"."</p>
            <Editor example="∵(⊂÷~1.) 1_2_4_5"/>
            <p>"Or, if you wanted to get the last element of each row of an array, you could use "<PrimCode prim=Rows/>"."</p>
            <Editor example="A ← [2_5_3 0_2_1 0_0_2]\nA\n≡(⊢⇌)A"/>
            <br/>
            <h2>"Dfns"</h2>
            <hr/>
            <p>"A dfn (pronounced \"dee-fun\") is a block of code that can bind names to single-character names."</p>
            <p>"Dfns are created with "<code>"{...}"</code>"."</p>
            <p>"Unlike functions created with "<code>"(...)"</code>", dfns are called imediately, much like a built-in function."</p>
            <p>"The number of arguments that a dfn takes is determined by which single-character lowercase ASCII names it refers to. This means you can use any of 26 names: "<code>"a"</code>", "<code>"b"</code>", "<code>"c"</code>", ... "<code>"z"</code>". A dfn that only refers to "<code>"a"</code>" takes 1 argument. A dfn that refers to "<code>"z"</code>" takes 26."</p>
            <p>"As an example, you could use a dfn to manually implement "<PrimCode prim=Flip/>" (don't do this)."</p>
            <Editor example="[1 2]\n[~ 1 2]\n[{b a} 1 2]"/>
            <p>"Dfns are particularly useful when you have to juggle three or more arguments."</p>
            <p>"If you tried to implement the "<a href="https://en.wikipedia.org/wiki/Quadratic_formula">quadratic formula</a>" with only stack operations, you would have a very hard time. Thankfully, dfns make it pretty simple."</p>
            <Editor example="Quadratic ← {÷ ×2a -b ⊟¯. √- ××4a c ⁿ2 b}\nQuadratic 1 2 0"/>
            <p>"Dfns are also required if you want to use "<PrimCode prim=Recur/>". The dfn that contains the "<PrimCode prim=Recur/>" will be the thing that recurs."</p>
            <p>"Here is a recursive fibonacci function."</p>
            <Editor example="{:⊡~(+ ↬-1a ↬-2a)_(a) <2a} 10"/>
            <p>"It is an error to access a dfn's argument outside of the dfn. This is only possible if you smuggle out the argument reference inside a function."</p>
            <Editor example=":{(a)}5"/>
        </div>
    }
}

#[component]
fn TutorialModules(cx: Scope) -> impl IntoView {
    use Primitive::*;
    view! { cx,
        <h1>"Modules"</h1>
        <p>"Modules are a way to organize your code in Uiua."</p>
        <br/>
        <h2>"Scopes"</h2>
        <hr/>
        <p>"Scopes are a way to create a temporary namespace for bindings that are only used in a small part of your code. Only the names that you want to escape a scope are usable outside it."</p>
        <p>"Scopes begin and end with triple hyphens "<code>"---"</code>". All names declared inside a scope are not available outside of it."</p>
        <Editor example="---\nfoo ← 5\n---\nfoo"/>
        <p>"Values pushed to the stack inside a scope remain on the stack after the scope ends."</p>
        <p>"You can bind values that were pushed to the stack inside an ended scope by using a "<code>"←"</code>" with nothing on the right side."</p>
        <Editor example="---\na ← 3\nb ← 5\n+ a b\n× a b\n---\nc ← \nd ←\nc_d"/>
        <p>"Note that scopes can only be created at the top level of a file, and they cannot be nested."</p>
        <br/>
        <h2>"Modules and "<PrimCode prim=Use/></h2>
        <hr/>
        <p><PrimCode prim=Use/>" is a very special function that extracts a function from a "<em>module</em>"."</p>
        <p>"A module is an array of "<em>"named"</em>" functions."</p>
        <p>"The only way to name a function is to bind it. Named functions can be put into arrays by stranding or surrounding the names with "<code>"()"</code>"."</p>
        <p><PrimCode prim=Use/>" takes the name of the function as a string and the module and returns the function."</p>
        <p>"If you want to give the function a name in the current scope, the name on the left side of the "<code>"←"</code>" must "<em>"not"</em>" be capitalized. (If you do capitalize it, you will end up with a function that returns a function.)"</p>
        <p>"The point of all of this is it allows you to export functions by name from a scope."</p>
        <Editor example=r#"---
PlusFive ← +5
Twin ← ⊟.
PlusFive_Twin
---
mymodule ←
twin ← use "twin" mymodule
plusfive ← use "PlusFive" mymodule

Twin PlusFive 3"#/>
        <br/>
        <h2><PrimCode prim=Io(IoOp::Import)/></h2>
        <hr/>
        <p>"Finally, we reach the point of all of this. You can import other files as scopes with "<PrimCode prim=Io(IoOp::Import)/>"."</p>
        <p>"The website's editor has an example file that you can import called "<code>"example.ua"</code>". Its contents is:"</p>
        <Editor example=EXAMPLE_UA/>
        <p>"You can import it with "<PrimCode prim=Io(IoOp::Import)/>" and then "<PrimCode prim=Use/>" to extract the functions."</p>
        <p>"By using "<PrimCode prim=Dup/>" on the imported module, you can repetedly extract functions from it. Notice the lack of a "<PrimCode prim=Dup glyph_only=true/>" after the last "<PrimCode prim=Use/>"."</p>
        <Editor example=r#"import "example.ua"
square ← use "square".
double ← use "double".
increment ← use "increment"

increment square double 5"#/>
        <p><PrimCode prim=Io(IoOp::Import)/>" only imports a given file once and caches the results. Subsequent imports of the same file (from anywhere) will not run the file's code again, but they will push its stack values again."</p>
        <p>"In this example, we make some code that prints a message and then generates a random number. We then write the code to a file and import it 3 times. Notice that the message is only printed once, and the same number is returned every time."</p>
        <Editor example=r#"code ← "print \"Loading module\"\nrand"
FWriteAll "test.ua" code
⍥(import "test.ua")3"#/>
    }
}

PDF=wasm-neovm-spec.pdf
TEX=wasm-neovm-spec.tex

.PHONY: all clean

all: $(PDF)

$(PDF): $(TEX)
	pdflatex -interaction=nonstopmode -halt-on-error $(TEX)

clean:
	rm -f $(PDF) *.aux *.log *.out *.toc

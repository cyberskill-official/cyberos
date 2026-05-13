# docs/Makefile — convert binary docs (PRD, SRS) to/from markdown.
#
# Markdown is the source of truth from 2026-05-14 onwards. Regenerate the
# docx files from markdown via `make docx`; convert any new docx back to
# markdown via `make md`.
#
# pandoc must be installed:
#   macOS:  brew install pandoc
#   linux:  apt-get install pandoc

PANDOC      := pandoc
PANDOC_FLAGS_MD := -f docx -t gfm --wrap=preserve

# Reference docx — used as a style template when generating docx from md.
# Update this when the desired styling changes; everything else inherits.
REFERENCE_DOCX := prd/PRD.docx

PRD_MD   := prd/PRD.md
PRD_DOCX := prd/PRD.docx
PRD_MEDIA := prd/media

SRS_MD   := srs/SRS.md
SRS_DOCX := srs/SRS.docx
SRS_MEDIA := srs/media

.PHONY: all docx md prd-md prd-docx srs-md srs-docx clean-media

# Default: ensure both markdown sources are up to date from their docx.
all: md

# ---- markdown → docx (regeneration; markdown is source of truth) -------

docx: prd-docx srs-docx

prd-docx: $(PRD_DOCX)
$(PRD_DOCX): $(PRD_MD)
	$(PANDOC) $< \
	    -f gfm -t docx \
	    --reference-doc=$(REFERENCE_DOCX) \
	    --resource-path=.:prd:srs \
	    -o $@
	@echo "  wrote $@"

srs-docx: $(SRS_DOCX)
$(SRS_DOCX): $(SRS_MD)
	$(PANDOC) $< \
	    -f gfm -t docx \
	    --reference-doc=$(REFERENCE_DOCX) \
	    --resource-path=.:prd:srs \
	    -o $@
	@echo "  wrote $@"

# ---- docx → markdown (initial conversion / re-import) ------------------

md: prd-md srs-md

prd-md:
	$(PANDOC) $(PRD_DOCX) $(PANDOC_FLAGS_MD) \
	    --extract-media=$(PRD_MEDIA) \
	    -o $(PRD_MD)
	@echo "  wrote $(PRD_MD) (+ media → $(PRD_MEDIA))"

srs-md:
	$(PANDOC) $(SRS_DOCX) $(PANDOC_FLAGS_MD) \
	    --extract-media=$(SRS_MEDIA) \
	    -o $(SRS_MD)
	@echo "  wrote $(SRS_MD) (+ media → $(SRS_MEDIA))"

clean-media:
	rm -rf $(PRD_MEDIA) $(SRS_MEDIA)

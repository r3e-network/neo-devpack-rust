// NeoVM subtarget feature definitions

def FeatureNeoVM : SubtargetFeature<"neovm", "HasNeoVM", "true",
                                    "Enable base NeoVM feature set">;

def FeatureTryCatch : SubtargetFeature<"try-catch", "HasTryCatch", "true",
                                       "Enable TRY/ENDTRY opcodes">;

def FeatureManifestHints : SubtargetFeature<"manifest-hints", "HasManifestHints", "true",
                                            "Enable manifest hint support">;

def FeatureBigIntegers : SubtargetFeature<"big-integers", "HasBigIntegers", "true",
                                          "Enable 256-bit arithmetic support">;

def : Processor<"neovm", NoItineraries,
                [FeatureNeoVM, FeatureTryCatch, FeatureManifestHints, FeatureBigIntegers]>;

def : Processor<"neovm-v1", NoItineraries,
                [FeatureNeoVM, FeatureTryCatch, FeatureManifestHints, FeatureBigIntegers]>;

def : Processor<"neovm-v2", NoItineraries,
                [FeatureNeoVM, FeatureTryCatch, FeatureManifestHints, FeatureBigIntegers]>;

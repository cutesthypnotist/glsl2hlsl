Shader "Converted/Discrete String SimulationImage"
{
    Properties
    {
        [Header(General)]
        _MainTex ("iChannel0", 2D) = "white" {}
        _SecondTex ("iChannel1", 2D) = "white" {}
        _ThirdTex ("iChannel2", 2D) = "white" {}
        _FourthTex ("iChannel3", 2D) = "white" {}


            _MainTex_1 ("Buffer 1 iChannel0", 2D) = "white" {}
            _SecondTex_1 ("Buffer 1 iChannel1", 2D) = "white" {}
            _ThirdTex_1 ("Buffer 1 iChannel2", 2D) = "white" {}
            _FourthTex_1 ("Buffer 1 iChannel3", 2D) = "white" {}

        _Mouse ("Mouse", Vector) = (0.5, 0.5, 0.5, 0.5)
        [ToggleUI] _GammaCorrect ("Gamma Correction", Float) = 1
        _Resolution ("Resolution (Change if AA is bad)", Range(1, 1024)) = 1
    }        CGINCLUDE

    #ifndef COMMAN_INCLUDE_BLOCK
    #define COMMAN_INCLUDE_BLOCK
        #define glsl_mod(x,y) (((x)-(y)*floor((x)/(y))))
        #define texelFetch(ch, uv, lod) tex2Dlod(ch, float4((uv).xy * ch##_TexelSize.xy + ch##_TexelSize.xy * 0.5, 0, lod))
        #define textureLod(ch, uv, lod) tex2Dlod(ch, float4(uv, 0, lod))
        #define iResolution float3(_Resolution, _Resolution, _Resolution)
        #define iFrame (floor(_Time.y / 60))
        #define iChannelTime float4(_Time.y, _Time.y, _Time.y, _Time.y)
        #define iDate float4(2020, 6, 18, 30)
        #define iSampleRate (44100)
    #endif       
                static int n_particles = 80;
            static float stiffness = 1.3;
            static float damping = 0.006;
            static float kick_strength = 0.005;
            float hash12(float2 p)
            {
                float h = dot(p, float2(127.1, 311.7));
                return frac(abs(sin(h)*43758.547));
            }

        ENDCG

    SubShader
    {


        Pass 
            {
                Name "1"

                CGPROGRAM
                #include "UnityCustomRenderTexture.cginc"
                #pragma target 5.0
                #pragma vertex CustomRenderTextureVertexShader
                #pragma fragment frag  
        
                #include "UnityCG.cginc"
                // Built-in properties
                sampler2D _MainTex_1;   float4 _MainTex_1_TexelSize;
                sampler2D _SecondTex_1; float4 _SecondTex_1_TexelSize;
                sampler2D _ThirdTex_1;  float4 _ThirdTex_1_TexelSize;
                sampler2D _FourthTex_1; float4 _FourthTex_1_TexelSize;                
                float4 _Mouse;
                float _GammaCorrect;
                float _Resolution;
                float _WorldSpace;
                float4 _Offset;

                //Def CRT Res
                #ifdef iResolution
                    #undef iResolution
                    #define iResolution float3(_CustomRenderTextureWidth, _CustomRenderTextureHeight, _Resolution)
                #endif
                // GLSL Compatability macros
                #ifndef COMMAN_INCLUDE_BLOCK
                #define COMMAN_INCLUDE_BLOCK
                    #define glsl_mod(x,y) (((x)-(y)*floor((x)/(y))))
                    #define texelFetch(ch, uv, lod) tex2Dlod(ch, float4((uv).xy * ch##_TexelSize.xy + ch##_TexelSize.xy * 0.5, 0, lod))
                    #define textureLod(ch, uv, lod) tex2Dlod(ch, float4(uv, 0, lod))
                    #define iResolution float3(_CustomRenderTextureWidth, _CustomRenderTextureHeight, _Resolution)
                    #define iFrame (floor(_Time.y / 60))
                    #define iChannelTime float4(_Time.y, _Time.y, _Time.y, _Time.y)
                    #define iDate float4(2020, 6, 18, 30)
                    #define iSampleRate (44100)
                #endif    
                #define iChannelResolution float4x4(                      \
                    _MainTex_1_TexelSize.z,   _MainTex_1_TexelSize.w,   0, 0, \
                    _SecondTex_1_TexelSize.z, _SecondTex_1_TexelSize.w, 0, 0, \
                    _ThirdTex_1_TexelSize.z,  _ThirdTex_1_TexelSize.w,  0, 0, \
                    _FourthTex_1_TexelSize.z, _FourthTex_1_TexelSize.w, 0, 0)                
                static v2f_customrendertexture vertex_output_1;
            float4 frag (v2f_customrendertexture __vertex_output) : SV_Target
            {
                vertex_output_1 = __vertex_output;
                float4 C = 0;
                float2 fragCoord = vertex_output_1.globalTexcoord.xy * iResolution.xy;
                C = float4(0., 0., 0., 0.);
                if (int(fragCoord.x)==0&&int(fragCoord.y)<n_particles)
                {
                    if (iFrame==0)
                    {
                        C.x = fragCoord.y/float(n_particles)-0.5;
                        if (int(fragCoord.y)>0&&int(fragCoord.y)<n_particles-1)
                        {
                            C.y = 0.4*sin(8.*3.14*C.x)*(C.x*C.x);
                        }
                        
                    }
                    else 
                    {
                        C = texelFetch(_MainTex_1, ((int2)fragCoord.xy), 0);
                        if (int(fragCoord.y)>0&&int(fragCoord.y)<n_particles-1)
                        {
                            if (length(C.x+0.5*iResolution.x/iResolution.y-_Mouse.z/iResolution.y)<2./float(n_particles))
                            {
                                C[3] -= kick_strength*(hash12(fragCoord+_Time.y+0.312349*fragCoord*_Time.y)+(C[2]-_Mouse.y/iResolution.y));
                            }
                            
                            float4 C_left = texelFetch(_MainTex_1, int2(fragCoord.x, fragCoord.y-1.), 0);
                            float4 C_right = texelFetch(_MainTex_1, int2(fragCoord.x, fragCoord.y+1.), 0);
                            C[3] -= stiffness*(C.y-(C_left.y+C_right.y)*0.5);
                            C[3] *= 1.-damping;
                            C.y += C[3];
                        }
                        
                    }
                }
                
                                return C;
            }

                ENDCG
            }        
            
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag

            #include "UnityCG.cginc"

            struct appdata
            {
                float4 vertex : POSITION;
                float2 uv : TEXCOORD0;
            };

            struct v2f
            {
                float2 uv : TEXCOORD0;
                float4 vertex : SV_POSITION;
            };

            // Built-in properties
            sampler2D _MainTex;   float4 _MainTex_TexelSize;
            sampler2D _SecondTex; float4 _SecondTex_TexelSize;
            sampler2D _ThirdTex;  float4 _ThirdTex_TexelSize;
            sampler2D _FourthTex; float4 _FourthTex_TexelSize;

            float4 _Mouse;
            float _GammaCorrect;
            float _Resolution;
            //Undef CRT Res
            #ifdef iResolution
                #undef iResolution
                #define iResolution float3(_Resolution, _Resolution, _Resolution)
            #endif
            // GLSL Compatability macros
            #define glsl_mod(x,y) (((x)-(y)*floor((x)/(y))))
            #define texelFetch(ch, uv, lod) tex2Dlod(ch, float4((uv).xy * ch##_TexelSize.xy + ch##_TexelSize.xy * 0.5, 0, lod))
            #define textureLod(ch, uv, lod) tex2Dlod(ch, float4(uv, 0, lod))
            #define iResolution float3(_Resolution, _Resolution, _Resolution)
            #define iFrame (floor(_Time.y / 60))
            #define iChannelTime float4(_Time.y, _Time.y, _Time.y, _Time.y)
            #define iDate float4(2020, 6, 18, 30)
            #define iSampleRate (44100)
            #define iChannelResolution float4x4(                      \
                _MainTex_TexelSize.z,   _MainTex_TexelSize.w,   0, 0, \
                _SecondTex_TexelSize.z, _SecondTex_TexelSize.w, 0, 0, \
                _ThirdTex_TexelSize.z,  _ThirdTex_TexelSize.w,  0, 0, \
                _FourthTex_TexelSize.z, _FourthTex_TexelSize.w, 0, 0)

            // Global access to uv data
            static v2f vertex_output;

            v2f vert (appdata v)
            {
                v2f o;
                o.vertex = UnityObjectToClipPos(v.vertex);
                o.uv =  v.uv;
                return o;
            }

            float4 frag (v2f __vertex_output) : SV_Target
            {
                vertex_output = __vertex_output;
                float4 F = 0;
                float2 fragCoord = vertex_output.uv * _Resolution;
                float2 uv = fragCoord.xy/iResolution.xy;
                uv.x -= 0.5;
                uv.x *= iResolution.x/iResolution.y;
                uv.y -= 0.5;
                uv *= 0.7;
                F = float4(0., 0., 0., 1.);
                for (int p = 0;p<n_particles; p++)
                {
                    float4 c = texelFetch(_MainTex, int2(0, p), 0);
                    F.r += smoothstep(0., 1., 1./length(uv-c.xy)*0.5*(0.001+abs(c[3])));
                }
                if (_Mouse.z>0.)
                {
                    F.b = 0.01/length(uv/0.7+float2(0.5*iResolution.x/iResolution.y, 0.5)-_Mouse.xy/iResolution.y);
                }
                
                if (_GammaCorrect) F.rgb = pow(F.rgb, 2.2);
                return F;
            }
            ENDCG
        }
    }
}
fileFormatVersion: 2
guid: 38CC1D2DCA52FA73694B20C1FF52C311
ShaderImporter:
    externalObjects: {}
    defaultTextures: []
    nonModifiableTextures: []
    userData: 
    assetBundleName: 
    assetBundleVariant: 
    %YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!21 &2100000
Material:
  serializedVersion: 6
  m_ObjectHideFlags: 0
  m_CorrespondingSourceObject: {fileID: 0}
  m_PrefabInstance: {fileID: 0}
  m_PrefabAsset: {fileID: 0}
  m_Name: TestMaterial
  m_Shader: {fileID: 4800000, guid: 38CC1D2DCA52FA73694B20C1FF52C311, type: 3}
  m_ShaderKeywords: 
  m_LightmapFlags: 4
  m_EnableInstancingVariants: 0
  m_DoubleSidedGI: 0
  m_CustomRenderQueue: -1
  stringTagMap: {}
  disabledShaderPasses: []
  m_SavedProperties:
    serializedVersion: 0
    m_TexEnvs:
    - _MainTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _SecondTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _ThirdTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _FourthTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _MainTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _SecondTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _ThirdTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _FourthTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    m_Floats:
    - _DstBlend: 0
    - _Mode: 0
    - _SrcBlend: 1
    - _ZWrite: 1
    - _GammaCorrect: 1
    m_Colors:
    - _Mouse: {r: 0.5, g: 0.5, b: 0.5, a: 0.5}
%YAML 1.1  
%TAG !u! tag:unity3d.com,2011:
--- !u!86 &8600000
CustomRenderTexture:
  m_ObjectHideFlags: 0
  m_CorrespondingSourceObject: {fileID: 0}
  m_PrefabInstance: {fileID: 0}
  m_PrefabAsset: {fileID: 0}
  m_Name: Buffer 1 CRT
  m_ImageContentsHash:
    serializedVersion: 2
    Hash: 00000000000000000000000000000000
  m_ForcedFallbackFormat: 4
  m_DownscaleFallback: 0
  serializedVersion: 3
  m_Width: 800
  m_Height: 450
  m_AntiAliasing: 1
  m_MipCount: -1
  m_DepthFormat: 0
  m_ColorFormat: 52
  m_MipMap: 0
  m_GenerateMips: 1
  m_SRGB: 0
  m_UseDynamicScale: 0
  m_BindMS: 0
  m_EnableCompatibleFormat: 1
  m_TextureSettings:
    serializedVersion: 2
    m_FilterMode: 0
    m_Aniso: 1
    m_MipBias: 0
    m_WrapU: 1
    m_WrapV: 1
    m_WrapW: 1
  m_Dimension: 2
  m_VolumeDepth: 1
  m_Material: {fileID: 2100000, guid: 38CC1D2DCA52FA73694B20C1FF52C311, type: 2}
  m_InitSource: 0
  m_InitMaterial: {fileID: 0}
  m_InitColor: {r: 1, g: 1, b: 1, a: 1}
  m_InitTexture: {fileID: 0}
  m_UpdateMode: 1
  m_InitializationMode: 2
  m_UpdateZoneSpace: 0
  m_CurrentUpdateZoneSpace: 0
  m_UpdateZones:
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  - updateZoneCenter: {x: 0.5, y: 0.5, z: 0.5}
    updateZoneSize: {x: 1, y: 1, z: 1}
    rotation: 0
    passIndex: -1
    needSwap: 1
  m_UpdatePeriod: 0
  m_ShaderPass: 1
  m_CubemapFaceMask: 4294967295
  m_DoubleBuffered: 1
  m_WrapUpdateZones: 0
%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!21 &2100000
Material:
  serializedVersion: 6
  m_ObjectHideFlags: 0
  m_CorrespondingSourceObject: {fileID: 0}
  m_PrefabInstance: {fileID: 0}
  m_PrefabAsset: {fileID: 0}
  m_Name: TestMaterial
  m_Shader: {fileID: 4800000, guid: 38CC1D2DCA52FA73694B20C1FF52C311, type: 3}
  m_ShaderKeywords: 
  m_LightmapFlags: 4
  m_EnableInstancingVariants: 0
  m_DoubleSidedGI: 0
  m_CustomRenderQueue: -1
  stringTagMap: {}
  disabledShaderPasses: []
  m_SavedProperties:
    serializedVersion: 0
    m_TexEnvs:
    - _MainTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _SecondTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _ThirdTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _FourthTex_0:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _MainTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _SecondTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _ThirdTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _FourthTex_1:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    m_Floats:
    - _DstBlend: 0
    - _Mode: 0
    - _SrcBlend: 1
    - _ZWrite: 1
    - _GammaCorrect: 1
    m_Colors:
    - _Mouse: {r: 0.5, g: 0.5, b: 0.5, a: 0.5}

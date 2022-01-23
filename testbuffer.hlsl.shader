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
    
                // GLSL Compatability macros
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
                float2 fragCoord = vertex_output_1.globalTexcoord.xy * _Resolution;
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



